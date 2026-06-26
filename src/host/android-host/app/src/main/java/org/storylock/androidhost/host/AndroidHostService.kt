package org.storylock.androidhost.host

import org.json.JSONObject

interface AndroidHostService {
  fun health(): JSONObject
  fun permissionSummary(): JSONObject
  fun questionBankStatus(): JSONObject
  fun storyTemplateStatus(): JSONObject
  fun storyTemplates(): JSONObject
  fun authorizationPolicy(request: JSONObject): JSONObject
  fun verify(request: JSONObject): JSONObject
  fun authorize(request: JSONObject): JSONObject
  fun execute(request: JSONObject): JSONObject
  fun revoke(request: JSONObject): JSONObject
}
