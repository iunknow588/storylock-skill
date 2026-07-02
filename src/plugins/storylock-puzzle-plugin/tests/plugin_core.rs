use serde_json::json;
use storylock_puzzle_plugin::{
    create_open_challenge_from_draft, normalize_answer, toggle_selection,
};

fn sample_draft() -> serde_json::Value {
    json!({
        "nodes": [
            {
                "nodeId": "q1",
                "question": "Question 1",
                "answerOptionsLocalOnly": [
                    { "text": "A" },
                    { "text": "B" },
                    { "text": "C" }
                ]
            },
            {
                "nodeId": "q2",
                "question": "Question 2",
                "answerOptionsLocalOnly": [
                    { "text": "D" },
                    { "text": "E" }
                ]
            }
        ]
    })
}

#[test]
fn builds_cells_from_draft() {
    let cells = create_open_challenge_from_draft(&sample_draft(), 2).expect("cells");
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0].cell_id, "q1");
    assert_eq!(cells[0].prompt_text, "Question 1");
    assert_eq!(cells[0].answer_options.len(), 9);
    assert_eq!(&cells[0].answer_options[..3], ["A", "B", "C"]);
    assert!(cells[0].answer_options[3..].iter().all(String::is_empty));
}

#[test]
fn toggles_selection_by_normalized_value() {
    let cells = create_open_challenge_from_draft(&sample_draft(), 2).expect("cells");
    let mut selections = vec![Vec::<String>::new(); 2];
    toggle_selection(&cells, &mut selections, 0, 0);
    assert_eq!(selections[0], vec!["A".to_string()]);
    toggle_selection(&cells, &mut selections, 0, 0);
    assert!(selections[0].is_empty());
}

#[test]
fn normalizes_answer_text() {
    assert_eq!(normalize_answer("  HeLLo "), "hello");
}
