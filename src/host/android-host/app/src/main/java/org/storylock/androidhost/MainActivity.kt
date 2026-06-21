package org.storylock.androidhost

import android.content.Intent
import android.os.Bundle
import android.view.Gravity
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.setPadding
import org.storylock.androidhost.host.HostBindingState
import org.storylock.androidhost.security.AttachedActivityConfirmation

class MainActivity : AppCompatActivity() {
  private lateinit var statusView: TextView
  private var registering = false

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    val app = application as StoryLockHostApplication
    val confirmation = app.localConfirmation
    if (confirmation is AttachedActivityConfirmation) {
      confirmation.attach(this)
    }

    statusView = TextView(this).apply {
      setPadding(40)
      textSize = 14f
    }

    val registerButton = Button(this).apply {
      text = "Register Host"
      setOnClickListener {
        registering = true
        renderStatus(app.registrationManager.currentState())
        app.registrationManager.refreshRegistration { state ->
          runOnUiThread {
            registering = false
            renderStatus(state)
          }
        }
      }
    }

    val refreshButton = Button(this).apply {
      text = "Refresh Status"
      setOnClickListener {
        renderStatus(app.registrationManager.currentState())
      }
    }

    val layout = LinearLayout(this).apply {
      orientation = LinearLayout.VERTICAL
      gravity = Gravity.TOP
      setPadding(24)
      addView(registerButton)
      addView(refreshButton)
      addView(statusView)
    }

    setContentView(
      ScrollView(this).apply {
        addView(layout)
      },
    )

    handleIntent(intent)
    renderStatus(app.registrationManager.currentState())
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    setIntent(intent)
    handleIntent(intent)
    val app = application as StoryLockHostApplication
    renderStatus(app.registrationManager.currentState())
  }

  private fun handleIntent(intent: Intent?) {
    val uri = intent?.data ?: return
    val app = application as StoryLockHostApplication
    app.registrationManager.applyDeepLink(uri)
  }

  private fun renderStatus(state: HostBindingState) {
    val app = application as StoryLockHostApplication
    val permissionSummary = app.storyLockPackageRepository.loadPermissionSummary()
    statusView.text = buildString {
      appendLine("StoryLock Android Host")
      appendLine()
      appendLine("Status: ${statusLabel(state)}")
      appendLine()
      appendLine("Identity: ${app.hostConfig.identityId}")
      appendLine("Device ID: ${state.deviceId}")
      appendLine("App Instance ID: ${state.appInstanceId}")
      appendLine("Host port: ${app.hostConfig.port}")
      appendLine("Server running: ${app.server.isRunning()}")
      appendLine("Permission objects: ${permissionSummary.permissionObjects}")
      appendLine()
      appendLine("Gateway: ${state.gatewayBaseUrl.ifBlank { "(not bound yet)" }}")
      appendLine("Preferred mode: ${state.preferredMode}")
      appendLine("Registration ID: ${state.registrationId ?: "(not registered)" }")
      appendLine("Relay poll URL: ${state.relayPollUrl ?: "(pending)" }")
      appendLine("Deep link: ${state.deepLink ?: "(none)" }")
      appendLine("Last registered: ${state.lastRegisteredAt ?: 0L}")
      appendLine("Last seen: ${state.lastSeenAt ?: 0L}")
      appendLine("Last error: ${state.lastError ?: "(none)" }")
    }
  }

  private fun statusLabel(state: HostBindingState): String {
    if (registering) {
      return "registering"
    }
    return when (state.displayStatus()) {
      "unbound" -> "not bound"
      "bound_not_registered" -> "bound, not registered"
      "online" -> "online"
      "offline" -> "offline"
      else -> state.displayStatus()
    }
  }
}
