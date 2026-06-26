package org.storylock.androidhost.host

import android.content.Context
import org.json.JSONObject

data class AndroidQuestionSet(
  val identityId: String,
  val schemaVersion: String,
  val questionSetVersion: String,
  val normalizationVersion: String,
  val questions: List<ChallengeCell>,
)

class AndroidQuestionSetRepository(
  private val context: Context,
  private val assetName: String = DEFAULT_ASSET_NAME,
) {
  fun loadActiveQuestionSet(): AndroidQuestionSet {
    val json = context.assets.open(assetName).bufferedReader(Charsets.UTF_8).use { it.readText() }
    val root = JSONObject(json)
    val schemaVersion = root.optString("schemaVersion", "android-local-question-bank-v1")
    val questionSetVersion = root.getString("questionSetVersion")
    val normalizationVersion = root.optString("normalizationVersion", "nfkc-lower-v1")
    val identityId = root.optString("identityId", "android-demo-001")
    require(questionSetVersion.isNotBlank()) { "Android question set version must not be blank" }
    require(normalizationVersion.isNotBlank()) { "Android normalization version must not be blank" }
    require(identityId.isNotBlank()) { "Android question set identityId must not be blank" }
    val questionsJson = root.getJSONArray("questions")
    val seenQuestionIds = linkedSetOf<String>()
    val questions = buildList {
      for (index in 0 until questionsJson.length()) {
        val item = questionsJson.getJSONObject(index)
        if (item.optString("status", "active") != "active") {
          continue
        }
        val questionId = item.getString("questionId").trim()
        val promptText = item.optString("promptText", item.optString("promptRef", questionId)).trim()
        val answer = item.getString("answer").trim()
        require(questionId.isNotEmpty()) { "Android question set contains a blank questionId at index $index" }
        require(promptText.isNotEmpty()) { "Android question set contains a blank promptText for $questionId" }
        require(answer.isNotEmpty()) { "Android question set contains a blank answer for $questionId" }
        require(seenQuestionIds.add(questionId)) { "Android question set contains duplicate questionId: $questionId" }
        add(
          ChallengeCell(
            cellId = "cell-${size + 1}",
            questionId = questionId,
            promptText = promptText,
            answer = answer,
            position = size + 1,
          ),
        )
      }
    }
    if (questions.size < 24) {
      throw IllegalStateException("Android question set must contain at least 24 active questions")
    }
    return AndroidQuestionSet(
      identityId = identityId,
      schemaVersion = schemaVersion,
      questionSetVersion = questionSetVersion,
      normalizationVersion = normalizationVersion,
      questions = questions,
    )
  }

  companion object {
    const val DEFAULT_ASSET_NAME = "storylock-question-set.json"
  }
}
