package org.storylock.androidhost.host

import fi.iki.elonen.NanoHTTPD
import fi.iki.elonen.NanoHTTPD.Response.Status
import org.json.JSONObject

class AndroidHostServer(
  private val config: AndroidHostConfig,
  private val hostService: AndroidHostService,
) : NanoHTTPD("127.0.0.1", config.port) {
  @Volatile
  private var started = false

  fun isRunning(): Boolean = started

  override fun start() {
    super.start(SOCKET_READ_TIMEOUT, false)
    started = true
  }

  override fun stop() {
    started = false
    super.stop()
  }

  override fun serve(session: IHTTPSession): Response {
    if (config.sharedSecret.isNotBlank()) {
      val received = session.headers["x-storylock-shared-secret"].orEmpty()
      if (received != config.sharedSecret) {
        return jsonResponse(
          Status.UNAUTHORIZED,
          JSONObject()
            .put("status", "error")
            .put("message", "unauthorized"),
        )
      }
    }

    return when {
      session.method == Method.GET && session.uri == "/health" -> {
        jsonResponse(Status.OK, hostService.health())
      }

      session.method == Method.GET && session.uri == "/permission-summary" -> {
        jsonResponse(Status.OK, hostService.permissionSummary())
      }

      session.method == Method.POST && session.uri == "/execute" -> {
        val body = readBody(session)
        val request = JSONObject(body)
        jsonResponse(Status.OK, hostService.execute(request))
      }

      else -> {
        jsonResponse(
          Status.NOT_FOUND,
          JSONObject()
            .put("status", "error")
            .put("message", "not found"),
        )
      }
    }
  }

  private fun readBody(session: IHTTPSession): String {
    val files = HashMap<String, String>()
    session.parseBody(files)
    return files["postData"].orEmpty()
  }

  private fun jsonResponse(status: Status, payload: JSONObject): Response {
    return newFixedLengthResponse(status, "application/json; charset=utf-8", payload.toString(2))
  }
}
