package org.storylock.androidhost.security

import androidx.fragment.app.FragmentActivity
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine

class BiometricPromptLocalUserConfirmation(
  private val activity: FragmentActivity,
) : LocalUserConfirmation {
  override suspend fun confirm(request: LocalConfirmationRequest): LocalConfirmationResult {
    val biometricManager = BiometricManager.from(activity)
    val authenticators = if (request.strongConfirmationRequired) {
      BiometricManager.Authenticators.BIOMETRIC_STRONG
    } else {
      BiometricManager.Authenticators.BIOMETRIC_STRONG or
        BiometricManager.Authenticators.DEVICE_CREDENTIAL
    }

    val capability = biometricManager.canAuthenticate(authenticators)
    if (capability != BiometricManager.BIOMETRIC_SUCCESS) {
      return LocalConfirmationResult(
        approved = false,
        failureType = "biometric_unavailable",
        reason = "biometric capability unavailable: $capability",
      )
    }

    return suspendCancellableCoroutine { continuation ->
      val executor = ContextCompat.getMainExecutor(activity)
      val callback = object : BiometricPrompt.AuthenticationCallback() {
        override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
          if (continuation.isActive) {
            continuation.resume(LocalConfirmationResult(approved = true))
          }
        }

        override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
          if (continuation.isActive) {
            continuation.resume(
              LocalConfirmationResult(
                approved = false,
                failureType = if (errorCode == BiometricPrompt.ERROR_NEGATIVE_BUTTON || errorCode == BiometricPrompt.ERROR_USER_CANCELED || errorCode == BiometricPrompt.ERROR_CANCELED) {
                  "biometric_cancelled"
                } else {
                  "biometric_failed"
                },
                reason = "biometric error $errorCode: $errString",
              ),
            )
          }
        }

        override fun onAuthenticationFailed() {
          // Keep waiting. Android may allow additional biometric attempts before final failure.
        }
      }

      val prompt = BiometricPrompt(activity, executor, callback)
      val promptInfo = BiometricPrompt.PromptInfo.Builder()
        .setTitle(request.title)
        .setSubtitle(request.subtitle)
        .setDescription(request.reason)
        .setAllowedAuthenticators(authenticators)
        .build()

      prompt.authenticate(promptInfo)
    }
  }
}
