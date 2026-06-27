use super::*;

fn prompt_value(request: &Value, key: &str, fallback: &str) -> String {
    request
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn question_answer_pairs_json(request: &Value) -> String {
    let request_pairs = request
        .get("questionAnswerPairs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    serde_json::to_string(&request_pairs).unwrap_or_else(|_| "[]".to_string())
}

fn story_llm_prompt(request: &Value) -> String {
    format!(
        "Return JSON only. Do not use markdown fences. Do not explain.\n\
         Required top-level keys: title, summary, storyPlot, memoryAnchors, chapters, questionPlan.\n\
         Requirements:\n\
         1. Use Simplified Chinese for user-facing text.\n\
         2. storyPlot must naturally connect the provided question and answer pairs.\n\
         3. memoryAnchors must contain 6 to 8 short anchor phrases.\n\
         4. chapters must contain exactly 3 items, each with id and purpose.\n\
         5. questionPlan must include at least targetCount.\n\
         6. The built-in three story templates are examples only, not final stories to copy.\n\
         7. Explicitly encourage the user to rewrite and redesign the story manually instead of relying on AI polishing.\n\
         8. For better security, the story should become more private, more strange, more counterintuitive, and less guessable by outsiders while still memorable.\n\
         9. Avoid generic, polished, school-essay, or formulaic plots.\n\
         10. The output should feel like a draft for further human rewriting, not a final answer.\n\
         theme={theme}\n\
         audience={audience}\n\
         tone={tone}\n\
         questionAnswerPairs={pairs}",
        theme = prompt_value(request, "theme", "StoryLock story"),
        audience = prompt_value(request, "audience", "StoryLock user"),
        tone = prompt_value(request, "tone", "clear and memorable"),
        pairs = question_answer_pairs_json(request),
    )
}

fn parse_llm_response_content(value: &Value) -> Result<Value> {
    let content = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let trimmed = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .unwrap_or(content)
        .trim()
        .trim_end_matches("```")
        .trim();
    serde_json::from_str(trimmed).context("external story LLM did not return valid JSON")
}

pub(crate) fn safe_story_slug(value: &str) -> String {
    let slug = sanitize_ref(value);
    if slug.is_empty() {
        short_id()
    } else {
        slug
    }
}

pub(crate) fn call_story_llm(config: &StoryLlmConfig, request: &Value) -> Result<Value> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let body = json!({
        "model": config.model,
        "messages": [
            {
                "role": "system",
                "content": "You are a strict JSON generator for a StoryLock story template."
            },
            {
                "role": "user",
                "content": story_llm_prompt(request)
            }
        ],
        "temperature": 0.7
    });
    let response = client
        .post(url)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .context("failed to call external story LLM")?
        .error_for_status()
        .context("external story LLM returned an error status")?;
    let value = response.json::<Value>()?;
    parse_llm_response_content(&value)
}

pub(crate) fn generate_local_story_framework(request: &Value) -> Value {
    let theme = prompt_value(request, "theme", "StoryLock story");
    let audience = prompt_value(request, "audience", "local StoryLock user");
    let tone = prompt_value(request, "tone", "clear and memorable");
    let slug = safe_story_slug(&theme);
    let story_plot = request
        .get("questionAnswerPairs")
        .and_then(Value::as_array)
        .filter(|pairs| !pairs.is_empty())
        .map(|pairs| {
            let fragments = pairs
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let question = item
                        .get("question")
                        .and_then(Value::as_str)
                        .unwrap_or("Unnamed question");
                    let answer = item
                        .get("answer")
                        .and_then(Value::as_str)
                        .unwrap_or("Missing answer");
                    format!(
                        "Question {} uses '{}' and its answer is '{}'. ",
                        index + 1,
                        question,
                        answer
                    )
                })
                .collect::<Vec<_>>()
                .join("");
            format!(
                "This draft was assembled from StoryLock question-answer pairs. {}It is only a starting point and should be rewritten by the user into a more private, stranger, and less guessable memory story.",
                fragments
            )
        });
    json!({
        "title": format!("{theme} story draft"),
        "summary": format!(
            "A {tone} starter draft for {audience}. It is an example only and should be manually rewritten into something more private, strange, and difficult for outsiders to guess."
        ),
        "storyPlot": story_plot.unwrap_or_else(|| format!(
            "This is a starter draft around '{theme}' for {audience} with a {tone} tone. It should be treated as an example only, then manually rewritten into a more private, stranger, less predictable story instead of relying on AI polishing."
        )),
        "memoryAnchors": [
            format!("{theme} opening"),
            "private clue",
            "strange image",
            "memory checkpoint",
            "unexpected turn",
            "final boundary"
        ],
        "chapters": [
            { "id": format!("{slug}-setup"), "purpose": "Introduce the private scene, the people, and the first unusual clue." },
            { "id": format!("{slug}-challenge"), "purpose": "Connect the protected details to memorable events, reactions, and odd choices." },
            { "id": format!("{slug}-resolution"), "purpose": "End with a result that is memorable to the user but difficult for outsiders to infer." }
        ],
        "questionPlan": {
            "targetCount": request.get("questionCount").and_then(Value::as_u64).unwrap_or(24),
            "grid": "StoryLock decides the final grid positions and answer layout after import.",
            "hostBoundary": "candidate_framework_only"
        }
    })
}

pub(crate) fn generate_story_framework(runtime: &WindowsHostRuntime, request: &Value) -> Value {
    let _ = runtime;
    if let Some(config) = story_llm_config() {
        if let Ok(framework) = call_story_llm(&config, request) {
            return framework;
        }
    }
    generate_local_story_framework(request)
}
