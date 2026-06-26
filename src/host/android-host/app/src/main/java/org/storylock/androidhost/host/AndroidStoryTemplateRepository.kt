package org.storylock.androidhost.host

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.max

data class AndroidStoryTemplate(
  val templateId: String,
  val language: String,
  val storyTitle: String,
  val summary: String,
  val storyPlot: String,
)

class AndroidStoryTemplateRepository(
  private val context: Context,
  private val assetName: String = DEFAULT_MANIFEST_ASSET_NAME,
) {
  private fun loadManifest(): JSONObject {
    return context.assets.open(assetName).bufferedReader(Charsets.UTF_8).use { reader ->
      JSONObject(reader.readText().trimStart('\uFEFF'))
    }
  }

  private fun loadDraftAsset(fileName: String): JSONObject {
    val assetPath = "$ASSET_DIR/$fileName"
    val draft = context.assets.open(assetPath).bufferedReader(Charsets.UTF_8).use { reader ->
      JSONObject(reader.readText().trimStart('\uFEFF'))
    }
    validateDraft(draft, fileName)
    return draft
  }

  fun loadTemplates(): JSONArray {
    val manifest = loadManifest()
    val items = manifest.optJSONArray("items") ?: JSONArray()
    val templates = JSONArray()
    for (index in 0 until items.length()) {
      val item = items.optJSONObject(index) ?: continue
      val fileName = item.optString("fileName")
      if (fileName.isBlank()) {
        continue
      }
      templates.put(loadDraftAsset(fileName))
    }
    return templates
  }

  fun templateCount(): Int = loadTemplates().length()

  fun defaultTemplateId(): String? = loadManifest().optString("defaultTemplateId").ifBlank { null }

  private fun validateDraft(draft: JSONObject, fileName: String) {
    require(draft.optString("templateId").isNotBlank()) { "story draft templateId must be non-empty: $fileName" }
    require(draft.optString("language").isNotBlank()) { "story draft language must be non-empty: $fileName" }
    require(draft.optString("storyTitle").isNotBlank()) { "story draft storyTitle must be non-empty: $fileName" }
    require(draft.optString("storyPlot").isNotBlank()) { "story draft storyPlot must be non-empty: $fileName" }
    val nodes = draft.optJSONArray("nodes")
      ?: throw IllegalArgumentException("story draft nodes must be an array: $fileName")
    require(nodes.length() == 24) { "story draft nodes must contain exactly 24 items: $fileName" }
    for (index in 0 until nodes.length()) {
      val node = nodes.optJSONObject(index)
        ?: throw IllegalArgumentException("story draft node must be an object at ${index + 1}: $fileName")
      require(node.optString("nodeId").isNotBlank()) { "story draft nodeId must be non-empty at ${index + 1}: $fileName" }
      require(node.optString("question").isNotBlank()) { "story draft question must be non-empty at ${index + 1}: $fileName" }
      val options = node.optJSONArray("answerOptionsLocalOnly")
        ?: throw IllegalArgumentException("story draft answerOptionsLocalOnly must be an array at ${index + 1}: $fileName")
      require(options.length() >= 2) { "story draft answer options must contain at least 2 items at ${index + 1}: $fileName" }
      require(max(options.length(), 0) <= 9) { "story draft answer options must contain at most 9 items at ${index + 1}: $fileName" }
    }
  }

  companion object {
    const val ASSET_DIR = "story-drafts"
    const val DEFAULT_MANIFEST_ASSET_NAME = "$ASSET_DIR/manifest.json"
    const val DEFAULT_ASSET_NAME = DEFAULT_MANIFEST_ASSET_NAME
  }
}
