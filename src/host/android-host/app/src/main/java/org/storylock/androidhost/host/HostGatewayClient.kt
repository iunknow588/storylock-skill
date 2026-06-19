package org.storylock.androidhost.host

import android.os.Build
import org.json.JSONObject
import org.storylock.androidhost.BuildConfig
import java.io.BufferedReader
import java.io.OutputStreamWriter
import java.net.HttpURLConnection
import java.net.URL

class HostGatewayClient(
  private val config: AndroidHostConfig,
) {
  fun registerHost(
    state: HostBindingState,
    health: JSONObject,
  ): JSONObject {
    val gatewayBaseUrl = state.gatewayBaseUrl.trimEnd('/')
    require(gatewayBaseUrl.isNotBlank()) { "gateway base url is required" }

    val payload = JSONObject()
      .put("bindingToken", state.bindingToken)
      .put("identityId", state.identityId)
      .put("deviceId", state.deviceId)
      .put("appInstanceId", state.appInstanceId)
      .put("preferredMode", state.preferredMode)
      .put(
        "host",
        JSONObject()
          .put("healthUrl", "http://127.0.0.1:${config.port}/health")
          .put("executeUrl", "http://127.0.0.1:${config.port}/execute"),
      )
      .put(
        "install",
        JSONObject()
          .put("versionName", BuildConfig.VERSION_NAME)
          .put("versionCode", BuildConfig.VERSION_CODE),
      )
      .put(
        "device",
        JSONObject()
          .put("model", Build.MODEL)
          .put("manufacturer", Build.MANUFACTURER)
          .put("sdkInt", Build.VERSION.SDK_INT),
      )
      .put(
        "reachability",
        JSONObject()
          .put("localHttp", true)
          .put("relayPolling", state.usesRelay())
          .put("deepLink", state.deepLink)
          .put("healthStatus", health),
      )
    return requestJson("$gatewayBaseUrl/local-host/register", "POST", payload)
  }

  fun pollRelay(state: HostBindingState): JSONObject {
    val pollUrl = state.relayPollUrl ?: "${state.gatewayBaseUrl.trimEnd('/')}/local-host/relay/poll"
    val payload = JSONObject()
      .put("deviceId", state.deviceId)
      .put("appInstanceId", state.appInstanceId)
    return requestJson(pollUrl, "POST", payload)
  }

  fun submitRelayResponse(state: HostBindingState, relayRequestId: String, response: JSONObject): JSONObject {
    val respondUrl = state.relayRespondUrl ?: "${state.gatewayBaseUrl.trimEnd('/')}/local-host/relay/respond"
    val payload = JSONObject()
      .put("relayRequestId", relayRequestId)
      .put("response", response)
    return requestJson(respondUrl, "POST", payload)
  }

  private fun requestJson(endpoint: String, method: String, body: JSONObject? = null): JSONObject {
    val connection = URL(endpoint).openConnection() as HttpURLConnection
    connection.requestMethod = method
    connection.connectTimeout = 10_000
    connection.readTimeout = 25_000
    connection.setRequestProperty("content-type", "application/json; charset=utf-8")
    if (config.sharedSecret.isNotBlank()) {
      connection.setRequestProperty("x-storylock-shared-secret", config.sharedSecret)
    }
    if (body != null) {
      connection.doOutput = true
      OutputStreamWriter(connection.outputStream, Charsets.UTF_8).use { writer ->
        writer.write(body.toString())
      }
    }

    val code = connection.responseCode
    val input = if (code in 200..299) connection.inputStream else connection.errorStream
    val text = input?.bufferedReader()?.use(BufferedReader::readText).orEmpty()
    if (text.isBlank()) {
      if (code in 200..299) {
        return JSONObject()
      }
      throw IllegalStateException("gateway returned status $code with empty body")
    }
    val parsed = JSONObject(text)
    if (code !in 200..299) {
      throw IllegalStateException(parsed.optString("message", "gateway returned status $code"))
    }
    return parsed
  }
}
