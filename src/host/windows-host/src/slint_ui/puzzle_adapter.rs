use super::*;
use storylock_puzzle_plugin::StoryLockChallengeCell;

pub(super) fn show_storylock_authorization_result(title: &str, message: &str, success: bool) {
    let title = wide_null(title);
    let message = wide_null(message);
    let icon = if success {
        MB_ICONINFORMATION
    } else {
        MB_ICONERROR
    };
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | icon,
        );
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(super) fn set_storylock_challenge_question(
    dialog: &StoryLockAuthorizationDialog,
    cells: &[StoryLockChallengeCell],
    selections: &[Vec<String>],
    index: usize,
) {
    let cell = cells.get(index);
    let option = |option_index: usize| -> SharedString {
        SharedString::from(
            cell.and_then(|cell| cell.answer_options.get(option_index))
                .map(String::as_str)
                .unwrap_or(""),
        )
    };
    let prompt = cell.map(|cell| cell.prompt_text.as_str()).unwrap_or("");
    let cell_id = cell.map(|cell| cell.cell_id.as_str()).unwrap_or("");
    dialog.set_current_index(index as i32);
    let selected_count = selections.get(index).map(Vec::len).unwrap_or_default();
    dialog.set_current_position(SharedString::from(format!(
        "{}/{} - 已选 {}",
        index + 1,
        cells.len(),
        selected_count
    )));
    let missing = cells
        .iter()
        .enumerate()
        .filter(|(cell_index, _)| {
            selections
                .get(*cell_index)
                .map(Vec::is_empty)
                .unwrap_or(true)
        })
        .map(|(cell_index, _)| (cell_index + 1).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let prompt_text = if missing.is_empty() {
        prompt.to_string()
    } else {
        format!("{prompt} | 未完成题号: {missing}")
    };
    dialog.set_current_prompt(SharedString::from(if cell_id.is_empty() {
        prompt_text
    } else {
        prompt_text
    }));
    dialog.set_selected_answer(SharedString::from(
        selections
            .get(index)
            .map(|answers| answers.join("; "))
            .unwrap_or_default(),
    ));
    dialog.set_option_1(option(0));
    dialog.set_option_2(option(1));
    dialog.set_option_3(option(2));
    dialog.set_option_4(option(3));
    dialog.set_option_5(option(4));
    dialog.set_option_6(option(5));
    dialog.set_option_7(option(6));
    dialog.set_option_8(option(7));
    dialog.set_option_9(option(8));
    let state = |option_index: usize| -> SharedString {
        let option_value = cell
            .and_then(|cell| cell.answer_options.get(option_index))
            .map(String::as_str)
            .unwrap_or("");
        let option_normalized = storylock_puzzle_plugin::normalize_answer(option_value);
        let selected = selections.get(index).is_some_and(|answers| {
            answers.iter().any(|answer| {
                storylock_puzzle_plugin::normalize_answer(answer) == option_normalized
            })
        });
        SharedString::from(if selected && !option_normalized.is_empty() {
            "correct"
        } else {
            "wrong"
        })
    };
    dialog.set_option_1_state(state(0));
    dialog.set_option_2_state(state(1));
    dialog.set_option_3_state(state(2));
    dialog.set_option_4_state(state(3));
    dialog.set_option_5_state(state(4));
    dialog.set_option_6_state(state(5));
    dialog.set_option_7_state(state(6));
    dialog.set_option_8_state(state(7));
    dialog.set_option_9_state(state(8));
}
