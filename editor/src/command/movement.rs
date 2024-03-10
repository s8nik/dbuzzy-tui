use crate::buffer::Buffer;

enum Direction {
    Up,
    Down,
    Back,
    Forward,
}

pub(super) fn move_forward(buffer: &mut Buffer) {
    move_impl(buffer, Direction::Forward)
}

pub(super) fn move_back(buffer: &mut Buffer) {
    move_impl(buffer, Direction::Back)
}

pub(super) fn move_up(buffer: &mut Buffer) {
    move_impl(buffer, Direction::Up)
}

pub(super) fn move_down(buffer: &mut Buffer) {
    move_impl(buffer, Direction::Down)
}

fn move_impl(buffer: &mut Buffer, direction: Direction) {
    match direction {
        Direction::Up => {
            if buffer.index > 0 {
                buffer.index -= 1;
                buffer.offset = buffer.offset.min(buffer.line_len_bytes());
            }
        }
        Direction::Down => {
            if buffer.index < buffer.len_lines() - 1 {
                buffer.index += 1;
                buffer.offset = buffer.offset.min(buffer.line_len_bytes());
            }
        }
        Direction::Back => {
            if buffer.offset > 0 {
                buffer.offset -= 1;
            } else if buffer.index > 0 {
                buffer.index -= 1;
                buffer.offset = buffer.line_len_bytes() - 1;
            }
        }
        Direction::Forward => {
            if buffer.offset < buffer.line_len_bytes() {
                buffer.offset += 1;
            } else if buffer.index < buffer.len_lines() - 1 {
                buffer.offset = 0;
                buffer.index += 1;
            }
        }
    }
}

pub(super) fn go_to_start_line(buffer: &mut Buffer) {
    buffer.index = 0;
    buffer.offset = 0;
}

pub(super) fn go_to_end_line(buffer: &mut Buffer) {
    buffer.index = buffer.len_lines() - 1;
    buffer.offset = 0;
}

pub(super) fn go_to_start_curr_line(buffer: &mut Buffer) {
    buffer.offset = 0
}

pub(super) fn go_to_end_curr_line(buffer: &mut Buffer) {
    buffer.offset = buffer.line_len_bytes() - 1;
}
