package org.storylock.androidhost.security

import android.app.AlertDialog
import android.text.InputType
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.activity.ComponentActivity
import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine

class ChallengePromptLocalUserConfirmation(
  private val activity: ComponentActivity,
) {
  suspend fun confirm(title: String, challengeItems: List<LocalChallengeItem>): LocalConfirmationResult {
    return suspendCancellableCoroutine { continuation ->
      val container = LinearLayout(activity).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(32, 24, 32, 8)
      }
      val inputs = linkedMapOf<String, EditText>()
      challengeItems.forEach { item ->
        val label = TextView(activity).apply {
          text = "${item.position}. ${item.promptText}"
        }
        val input = EditText(activity).apply {
          inputType = InputType.TYPE_CLASS_TEXT
          hint = "Enter answer for ${item.cellId}"
        }
        inputs[item.cellId] = input
        container.addView(label)
        container.addView(input)
      }
      val scrollView = ScrollView(activity).apply {
        addView(container)
      }
      val dialog = AlertDialog.Builder(activity)
        .setTitle(title)
        .setMessage("Please complete the local challenge before continuing.")
        .setView(scrollView)
        .setCancelable(false)
        .setPositiveButton("Confirm") { _, _ ->
          val answers = inputs.mapValues { (_, input) ->
            input.text?.toString()?.trim().orEmpty()
          }
          val approved = challengeItems.all { item ->
            answers[item.cellId]?.equals(item.expectedAnswer.trim(), ignoreCase = true) == true
          }
          if (continuation.isActive) {
            continuation.resume(
              if (approved) {
                LocalConfirmationResult(
                  approved = true,
                  challengeAnswers = answers,
                )
              } else {
                LocalConfirmationResult(
                  approved = false,
                  failureType = "challenge_failed",
                  reason = "challenge answer mismatch",
                  challengeAnswers = answers,
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
                failureType = "challenge_cancelled",
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
