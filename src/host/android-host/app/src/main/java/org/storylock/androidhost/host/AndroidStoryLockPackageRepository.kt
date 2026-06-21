package org.storylock.androidhost.host

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

data class AndroidPermissionSummary(
  val packageId: String?,
  val resources: Int,
  val permissionObjects: Int,
  val permissionSummary: JSONObject,
)

class AndroidStoryLockPackageRepository(
  private val context: Context,
  private val resourceCatalogAssetName: String = DEFAULT_RESOURCE_CATALOG_ASSET_NAME,
) {
  fun loadPermissionSummary(): AndroidPermissionSummary {
    val catalog = context.assets.open(resourceCatalogAssetName).bufferedReader(Charsets.UTF_8).use { reader ->
      JSONObject(reader.readText().trimStart('\uFEFF'))
    }
    val resources = catalog.optJSONArray("resources") ?: JSONArray()
    val items = JSONArray()
    for (resourceIndex in 0 until resources.length()) {
      val resource = resources.optJSONObject(resourceIndex) ?: continue
      val bindings = resource.optJSONArray("bindings") ?: JSONArray()
      for (bindingIndex in 0 until bindings.length()) {
        val binding = bindings.optJSONObject(bindingIndex) ?: continue
        val objectMeta = binding.optJSONObject("objectMeta") ?: JSONObject()
        val objectKind = objectMeta.optString("objectKind", "secret")
        val sensitivity = objectMeta.optString("sensitivity", "private")
        items.put(
          JSONObject()
            .put("resourceId", resource.optString("resourceId"))
            .put("resourceKind", resource.optString("resourceKind"))
            .put("providerId", resource.optString("providerId"))
            .put("displayName", resource.optString("displayName", resource.optString("resourceId")))
            .put("role", binding.optString("role"))
            .put("objectId", binding.optString("objectId"))
            .put("objectKind", objectKind)
            .put("sensitivity", sensitivity)
            .put("action", permissionAction(objectKind))
            .put("challengePolicy", permissionChallengePolicy(sensitivity))
            .put("requiredGridCount", permissionRequiredGridCount(sensitivity)),
        )
      }
    }
    val summary = JSONObject().put("items", items)
    return AndroidPermissionSummary(
      packageId = null,
      resources = resources.length(),
      permissionObjects = items.length(),
      permissionSummary = summary,
    )
  }

  private fun permissionAction(objectKind: String): String {
    return when (objectKind) {
      "private_key", "signing_key" -> "sign"
      "password" -> "password_fill"
      else -> "read"
    }
  }

  private fun permissionChallengePolicy(sensitivity: String): String {
    return when (sensitivity) {
      "secret", "high" -> "high"
      else -> "medium"
    }
  }

  private fun permissionRequiredGridCount(sensitivity: String): Int {
    return when (sensitivity) {
      "secret", "high" -> 12
      else -> 6
    }
  }

  companion object {
    const val DEFAULT_RESOURCE_CATALOG_ASSET_NAME = "storylock-resource-catalog.json"
  }
}
