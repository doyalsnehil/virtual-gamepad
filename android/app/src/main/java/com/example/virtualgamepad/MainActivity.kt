package com.example.virtualgamepad

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.SystemBarStyle
import android.graphics.Color
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import com.example.virtualgamepad.theme.VirtualGamepadTheme
import com.example.virtualgamepad.data.NetworkManager
import com.example.virtualgamepad.data.LayoutManager

import android.os.VibrationEffect
import android.os.Vibrator
import android.content.Context

class MainActivity : ComponentActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    NetworkManager.init(applicationContext)
    LayoutManager.init(applicationContext)

    val vibrator = getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
    NetworkManager.onRumble = { durationMs ->
        if (durationMs > 0) {
            vibrator.vibrate(VibrationEffect.createOneShot(durationMs, VibrationEffect.DEFAULT_AMPLITUDE))
        }
    }

    enableEdgeToEdge(
        statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
        navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT)
    )
    setContent {
      VirtualGamepadTheme { 
          Surface(
              modifier = Modifier.fillMaxSize(), 
              color = MaterialTheme.colorScheme.background
          ) { 
              MainNavigation() 
          } 
      }
    }
  }
}
