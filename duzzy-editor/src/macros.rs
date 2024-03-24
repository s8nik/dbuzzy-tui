#[macro_export]
macro_rules! set_cursor {
    ($buffer:expr, index $op:tt $value:expr) => {{
        match stringify!($op) {
            "=" => $buffer.pos.index = $value,
            "+=" => $buffer.pos.index += $value,
            "-=" => $buffer.pos.index -= $value,
            _ => unreachable!(),
        };
    }};
    ($buffer:expr, offset $op:tt $value:expr) => {{
        match stringify!($op) {
            "=" => $buffer.pos.offset = $value,
            "+=" => $buffer.pos.offset += $value,
            "-=" => $buffer.pos.offset -= $value,
            _ => unreachable!(),
        };
    }};
    ($buffer:expr, $pos:expr) => {{
        $buffer.pos = $pos;
    }};
}
