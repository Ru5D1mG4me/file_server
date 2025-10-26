fn ceil(num1: u64, num2: u64) -> u32 {
    if num1 % num2 != 0 {
        return (num1 / num2 + 1) as u32;
    }

    (num1 / num2) as u32
}