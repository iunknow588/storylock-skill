package org.storylock.androidhost.security

import android.app.AlertDialog
import android.text.InputType
import android.widget.EditText
import androidx.activity.ComponentActivity
import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine

class ChallengePromptLocalUserConfirmation(
  private val activity: ComponentActivity,
) {
  suspend fun confirm(prompt: String, expectedAnswer: String): LocalConfirmationResult {
    return suspendCancellableCoroutine { continuation ->
      val input = EditText(activity).apply {
        inputType = InputType.TYPE_CLASS_TEXT
        hint = "Enter answer"
      }
      val dialog = AlertDialog.Builder(activity)
        .setTitle("StoryLock Challenge")
        .setMessage(prompt)
        .setView(input)
        .setCancelable(false)
        .setPositiveButton("Confirm") { _, _ ->
          val actual = input.text?.toString()?.trim().orEmpty()
          val approved = actual.equals(expectedAnswer.trim(), ignoreCase = true)
          if (continuation.isActive) {
            continuation.resume(
              if (approved) {
                LocalConfirmationResult(approved = true)
              } else {
                LocalConfirmationResult(
                  approved = false,
                  reason = "challenge answer mismatch",
                )
              },
            )
          }
        }
        .setNegativeButton("Cancel") { _, _ ->
          if (continuation.isActive) {
            continuation.resume(
              LocalConfirmationResult(
                approved = false,
                reason = "challenge cancelled by user",
              ),
            )
          }
        }
        .create()

      dialog.show()

      continuation.invokeOnCancellation {
        dialog.dismiss()
      }
    }
  }
}
