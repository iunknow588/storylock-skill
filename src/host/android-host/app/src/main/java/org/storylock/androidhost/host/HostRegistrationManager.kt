package org.storylock.androidhost.host

import android.net.Uri
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.json.JSONObject

class HostRegistrationManager(
  private val config: AndroidHostConfig,
  private val bindingStore: HostBindingStore,
  private val gatewayClient: HostGatewayClient,
  private val hostService: StoryLockAndroidHostService,
) {
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
  @Volatile
  private var relayJob: Job? = null

  fun currentState(): HostBindingState = bindingStore.current(config)

  fun healthJson(): JSONObject {
    val state = currentState()
    return JSONObject()
      .put("deviceId", state.deviceId)
      .put("appInstanceId", state.appInstanceId)
      .put("gatewayBaseUrl", state.gatewayBaseUrl)
      .put("preferredMode", state.preferredMode)
      .put("registrationId", state.registrationId)
      .put("relayPollUrl", state.relayPollUrl)
      .put("registered", state.registrationId != null)
      .put("lastRegisteredAt", state.lastRegisteredAt)
      .put("lastSeenAt", state.lastSeenAt)
      .put("lastError", state.lastError)
    }

  fun applyDeepLink(uri: Uri): HostBindingState {
    val state = bindingStore.applyDeepLink(uri, config)
    ensureStarted()
    refreshRegistration()
    return state
  }

  fun refreshRegistration(onComplete: ((HostBindingState) -> Unit)? = null) {
    scope.launch {
      val state = try {
        registerHost(force = true)
      } catch (error: Exception) {
        bindingStore.markError(config, error.message ?: "registration failed")
      }
      onComplete?.invoke(state)
    }
  }

  fun ensureStarted() {
    if (relayJob != null) {
      return
    }
    relayJob = scope.launch {
      while (isActive) {
        val state = currentState()
        if (!state.hasGateway() || !state.usesRelay()) {
          delay(3_000)
          continue
        }

        try {
          val activeState = registerHost(force = shouldRefreshRegistration(state))
          val poll = gatewayClient.pollRelay(activeState)
          when (poll.optString("status")) {
            "ok" -> {
              val relayRequestId = poll.getString("relayRequestId")
              val request = poll.getJSONObject("request")
              val response = hostService.execute(request)
              gatewayClient.submitRelayResponse(activeState, relayRequestId, response)
              bindingStore.markSeen(config)
            }

            "idle" -> {
              bindingStore.markSeen(config)
              delay(500)
            }

            else -> {
              bindingStore.markError(config, "unexpected relay poll status")
              delay(2_000)
            }
          }
        } catch (error: Exception) {
          bindingStore.markError(config, error.message ?: "relay loop failed")
          delay(2_000)
        }
      }
    }
  }

  private fun shouldRefreshRegistration(state: HostBindingState): Boolean {
    val lastRegisteredAt = state.lastRegisteredAt ?: return true
    return System.currentTimeMillis() - lastRegisteredAt >= 60_000
  }

  private fun registerHost(force: Boolean): HostBindingState {
    val current = currentState()
    if (!current.hasGateway()) {
      return bindingStore.markError(
        config,
        "gateway is not bound; open /android-host/bind from the website or rebuild with STORYLOCK_GATEWAY_URL",
      )
    }
    if (!force && current.registrationId != null && !shouldRefreshRegistration(current)) {
      return current
    }
    val response = gatewayClient.registerHost(current, hostService.health())
    val registration = response.getJSONObject("registration")
    val relay = response.getJSONObject("relay")
    return bindingStore.updateRegistration(
      config = config,
      registrationId = registration.optString("registrationId"),
      relayUrl = relay.optString("relayUrl"),
      relayPollUrl = relay.optString("pollUrl"),
      relayRespondUrl = relay.optString("respondUrl"),
      deepLink = registration.optString("deepLink").ifBlank { current.deepLink },
    )
  }
}
