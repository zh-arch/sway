script;

enum SignedNum {
    Positive: u64,
    Negative: u64,
}

struct OpName {
    val: u64
}

fn main(ops: [(OpName, SignedNum); 2]) -> u64 {
    let mut result = 0;
    
    let mut i = 0;
    while i < 2 {
        let (op, val) = ops[i];

        if op.val == 0 {
            match val {
                SignedNum::Positive(v) => {
                    result = v;
                }
                SignedNum::Negative(_) => {
                    revert(0);
                }
            }
        } else if op.val == 1 {
            match val {
                SignedNum::Positive(v) => {
                    result += v;
                }
                SignedNum::Negative(v) => {
                    result -= v;
                }
            }
        } else {
            revert(1);
        }

        i += 1;
    }

    result
}
