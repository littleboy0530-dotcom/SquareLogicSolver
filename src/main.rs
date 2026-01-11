//셀 타입 열거형
#[derive(PartialEq, Clone, Debug)]
enum Celltype {
    NoInfo,
    X,
    O,
}

//현재 풀이 상태
enum SolveState{
    Progress,
    Stuck,
    Solved,
}

//보드 구조체
struct Board {
    cells: Vec<Vec<Celltype>>,
}

//보드 구조체 -생성자
impl Board {
    fn new(size: usize) -> Self {
        let cells = vec![vec![Celltype::NoInfo; size]; size];
        Board { cells }
    }

    fn is_complete(&self) -> bool {
        for line in self.cells.iter() {
            if line.contains(&Celltype::NoInfo){
                return false
            }
        }

        true
    }

    fn display(&self) {
        for line in self.cells.iter() {
            for cell in line {
                match cell {
                    Celltype::O => print!("#"),
                    Celltype::X => print!("."),
                    Celltype::NoInfo => print!("?"),
                }
            }

            println!();
        }
    }
}

//힌트 구조체
#[derive(Debug)]
struct Hint {
    index: isize,
    numbers: Vec<usize>,
    remaining_sum: Vec<usize>,
    candidates: Vec<u32>,
}

impl Hint {
    //힌트 구조체 -생성자
    fn new(index: isize, numbers: Vec<usize>) -> Self {
        let remain = calculate_remain_block_len(&numbers);
        Hint {
            index,
            numbers,
            remaining_sum: remain,
            candidates: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn display_candidate(&self, size: usize) {
        println!("{}번의 후보들", self.index);
        for &cnadidate in &self.candidates {
            println!("{:0width$b}", cnadidate, width =size);
        }
    }
}

//남은 블록들의 최소 길이 계산
fn calculate_remain_block_len(hint_num: &[usize]) -> Vec<usize> {
    let mut remain = vec![0; hint_num.len() +1];
    for i in (0..hint_num.len()).rev() {
        remain[i] = remain[i+1] +hint_num[i];
    }

    remain
}

#[inline]
fn paint_block(start: usize, length: usize) -> u32 {
    ((1u32 << length) -1) << start
}

//커서의 비트가 0이면 false 1이면 true를 반환하는 함수
#[inline]
fn check_bit(target_bits:u32, cursor: u32) -> bool {
    (target_bits & cursor) != 0
}

//핵심 구현
impl Hint {
    //재귀 탐색으로 가능한 모든 후보를 생성
    fn generate_patterns (
        &mut self,
        block_index: usize,
        next_free_cell: usize,
        line_length: usize,
        current_pattern: u32
    ){
        let blocks = &self.numbers;

        if block_index == blocks.len() {
            self.candidates.push(current_pattern);
            return;
        }

        let current_block_len = blocks[block_index];

        //남은 블록이 채워야 하는 최소 공간
        let remaining_blocks = blocks.len() -block_index -1;
        let min_required_after = if remaining_blocks > 0 {
            self.remaining_sum[block_index +1] +remaining_blocks
        }else {
            0
        };

        //현재 블록이 시작할수 있는 마지막 위치
        let last_start_pos = line_length -current_block_len -min_required_after;


        for start_pos in next_free_cell..=last_start_pos {
            let block_bit = paint_block(start_pos, current_block_len);
            let new_pattern = current_pattern | block_bit;

            self.generate_patterns(block_index +1,
            start_pos +current_block_len +1,
            line_length, new_pattern);
        }
    }
}

impl Board {
    //인덱스 정보로 해당 줄을 복사해오는 헬퍼 함수
    fn copy_by_index(&self, idx: isize) -> Vec<Celltype> {
        let mut result = Vec::new();

        if idx > 0 {
            let index = idx as usize -1;
            result = self.cells[index].clone();
        }else {
            let index = idx.abs() as usize -1;
            for i in 0..self.cells.len() {
                result.push(self.cells[i][index].clone())
            }
        }

        result
    }

    //보드 정보를 기반으로 후보군을 필터링하는 메서드
    fn filtering_candidate(&self, hint: &mut Hint){
        let line = self.copy_by_index(hint.index);

        if line.iter().all(|val| *val == Celltype::NoInfo) {
            return
        }

        let mut cursor = 1u32;
        for cell_type in line.iter(){
            match cell_type {
                Celltype::O => {
                    hint.candidates.retain(|&bits| check_bit(bits, cursor));
                },
                Celltype::X => {
                    hint.candidates.retain(|&bits| !check_bit(bits, cursor));
                },
                Celltype::NoInfo => {
                    //아무 동작도 하지 않음
                }
            }

            cursor <<= 1;
        }
    }

    //모든 후보군의 공통된 정보를 보드에 반영 TODO:변경 여부를 반환하도록 만들기
    fn reflection_candidate_info(&mut self, hint: &Hint) -> bool{
        let mut changed = false;

        if hint.candidates.is_empty() {
            println!("{}번 힌트", hint.index);
            print!("{:?}", self.copy_by_index(hint.index));
            panic!("모든 후보 필터링 됨");
        }

        let size = self.cells.len();

        let mut all_o:u32 = (1u32 << size ) -1;
        let mut all_x:u32 = 0u32;

        for candidate in hint.candidates.iter() {
            all_o &= *candidate;
            all_x |= *candidate;
        }

        all_x = !all_x;

        if all_o & all_x != 0 {
            panic!("{}번 힌트에 모순이 존재\n 모두 O: {}\n 모두 X: {}\n", hint.index, all_o, all_x);
        }

        let safe_hint_idx = hint.index.unsigned_abs() as usize -1;

        if hint.index > 0 /*가로열 힌트*/{
            //정보가 없는 칸만 순회
            for idx in 0..size {
                if self.cells[safe_hint_idx][idx] == Celltype::NoInfo {
                    let cursor = 1u32 << idx;
                    if cursor & all_o != 0 {
                        self.cells[safe_hint_idx][idx] = Celltype::O;
                        changed = true;
                    }else if cursor & all_x != 0 {
                        self.cells[safe_hint_idx][idx] = Celltype::X;
                        changed = true;
                    }
                }
            }
        }else if hint.index < 0 /*세로열 힌트*/{
            //정보가 없는 칸만 순회
            for row in 0..size {
                if self.cells[row][safe_hint_idx] == Celltype::NoInfo {
                    let cursor = 1u32 << row;
                    if cursor & all_o != 0 {
                        self.cells[row][safe_hint_idx] = Celltype::O;
                        changed = true;
                    }else if cursor & all_x != 0 {
                        self.cells[row][safe_hint_idx] = Celltype::X;
                        changed = true;
                    }
                }
            }
        }

        changed
    }
}

fn propagate_logic(board: &mut Board, 
    row_hints: &mut Vec<Hint>, col_hints: &mut Vec<Hint>
) -> SolveState {
    loop {
        let mut changed_flag = false;

        for hint in row_hints.iter_mut() {
            board.filtering_candidate(hint);
            if board.reflection_candidate_info(hint) {
                changed_flag = true;
                board.display();
            }
        }

        for hint in col_hints.iter_mut() {
            board.filtering_candidate(hint);
            if board.reflection_candidate_info(hint) {
                changed_flag = true;
                board.display();
            }
        }

        if board.is_complete() {
            return SolveState::Solved;
        }

        if changed_flag == false {
            return SolveState::Stuck;
        }
    }
}

// fn dfs_solve(row_hint:Vec<Hint>, col_hint:Vec<Hint>, size: usize) {
//     for i in 0..size {
//         let row_candidate_size = row_hint[i].candidates.len();
//         let col_candidate_size = col_hint[i].candidates.len();
//     }
// }

fn main() {
    let size = 20;
    
    let mut row_hints = vec![
        Hint::new(1,vec![6]),
        Hint::new(2,vec![5]),
        Hint::new(3,vec![7]),
        Hint::new(4,vec![11]),
        Hint::new(5,vec![13]),
        Hint::new(6,vec![11,2]),
        Hint::new(7,vec![17]),
        Hint::new(8,vec![7,8]),
        Hint::new(9,vec![5,8]),
        Hint::new(10,vec![5,2,3]),
        Hint::new(11,vec![4,2,3]),
        Hint::new(12,vec![4,1,2,1]),
        Hint::new(13,vec![4,1,3]),
        Hint::new(14,vec![3,1,5]),
        Hint::new(15,vec![5,3,1,1]),
        Hint::new(16,vec![5,3,1]),
        Hint::new(17,vec![6,1]),
        Hint::new(18,vec![4]),
        Hint::new(19,vec![3]),
        Hint::new(20,vec![3]),
        // Hint::new(21,vec![1,2,1,8,4,4]),
        // Hint::new(22,vec![1,1,1,1,2,1,4,3,1]),
        // Hint::new(23,vec![1,4,1,1,1]),
        // Hint::new(24,vec![1,2,7,16]),
        // Hint::new(25,vec![2,5,1,3,2,2,2,1]),
        // Hint::new(26,vec![3,3,2,1,1,2,2,5,1]),
        // Hint::new(27,vec![4,2,2,3,3,3,2]),
        // Hint::new(28,vec![11,2,3,5,3]),
        // Hint::new(29,vec![29]),
        // Hint::new(30,vec![29]),
    ];

    let mut col_hints = vec![
        Hint::new(-1,vec![4]),
        Hint::new(-2,vec![7]),
        Hint::new(-3,vec![9]),
        Hint::new(-4,vec![1,11]),
        Hint::new(-5,vec![1,6,2]),
        Hint::new(-6,vec![2,6,7]),
        Hint::new(-7,vec![8,2,3]),
        Hint::new(-8,vec![8,1,3]),
        Hint::new(-9,vec![7,1,1,2]),
        Hint::new(-10,vec![6,1,2,4]),
        Hint::new(-11,vec![5,3,6]),
        Hint::new(-12,vec![5,3,2,2]),
        Hint::new(-13,vec![8,1,1]),
        Hint::new(-14,vec![6]),
        Hint::new(-15,vec![2,3]),
        Hint::new(-16,vec![6,3]),
        Hint::new(-17,vec![4,2,1]),
        Hint::new(-18,vec![2,4]),
        Hint::new(-19,vec![2,2]),
        Hint::new(-20,vec![1,1]),
        // Hint::new(-21,vec![1,1,1,5,1,5]),
        // Hint::new(-22,vec![1,5,1,2,2,1,5]),
        // Hint::new(-23,vec![1,1,3,2,2,1,4]),
        // Hint::new(-24,vec![2,2,1,1,2,5,3,3]),
        // Hint::new(-25,vec![12,2,1,4,2]),
        // Hint::new(-26,vec![1,2,2,2,2,5,1,2,2]),
        // Hint::new(-27,vec![1,1,2,1,3,4,2]),
        // Hint::new(-28,vec![2,1,3,2,2,1,3,3]),
        // Hint::new(-29,vec![7,4,3,4,4]),
        // Hint::new(-30,vec![11,2,4,1]),
    ];

    if row_hints.len() != size || col_hints.len() != size || row_hints.len() != col_hints.len() {
        panic!("힌트길이 불일치");
    }

    let mut board = Board::new(size);

    for hint in row_hints.iter_mut() {
        hint.generate_patterns(0,0,size,0);
    }

    for hint in col_hints.iter_mut() {
        hint.generate_patterns(0,0,size,0);
    }

    println!("후보 생성 완료");

    match propagate_logic(&mut board, &mut row_hints, &mut col_hints) {
        SolveState::Solved => {
            println!("완성!");
            board.display();
        },
        SolveState::Stuck => {
            println!("DFS 필요");
            board.display();
        },
        SolveState::Progress => {
            panic!("UNREACHABLE!!");
        }
    }
}

// Hint::new(1,vec![]),
        // Hint::new(2,vec![]),
        // Hint::new(3,vec![]),
        // Hint::new(4,vec![]),
        // Hint::new(5,vec![]),
        // Hint::new(6,vec![]),
        // Hint::new(7,vec![]),
        // Hint::new(8,vec![]),
        // Hint::new(9,vec![]),
        // Hint::new(10,vec![]),
        // Hint::new(11,vec![]),
        // Hint::new(12,vec![]),
        // Hint::new(13,vec![]),
        // Hint::new(14,vec![]),
        // Hint::new(15,vec![]),
        // Hint::new(16,vec![]),
        // Hint::new(17,vec![]),
        // Hint::new(18,vec![]),
        // Hint::new(19,vec![]),
        // Hint::new(20,vec![]),
//         Hint::new(21,vec![]),
//         Hint::new(22,vec![]),
//         Hint::new(23,vec![]),
//         Hint::new(24,vec![]),
//         Hint::new(25,vec![]),
//         Hint::new(26,vec![]),
//         Hint::new(27,vec![]),
//         Hint::new(28,vec![]),
//         Hint::new(29,vec![]),
//         Hint::new(30,vec![]),