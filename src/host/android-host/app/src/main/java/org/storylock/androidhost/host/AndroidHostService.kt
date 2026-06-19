package org.storylock.androidhost.host

import org.json.JSONObject

interface AndroidHostService {
  fun health(): JSONObject
  fun execute(request: JSONObject): JSONObject
}
