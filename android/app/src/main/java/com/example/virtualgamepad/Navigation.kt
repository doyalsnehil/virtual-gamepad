package com.example.virtualgamepad

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.ui.NavDisplay
import com.example.virtualgamepad.ui.home.HomeScreen
import com.example.virtualgamepad.ui.play.PlayScreen
import com.example.virtualgamepad.ui.home.HomeScreen
import com.example.virtualgamepad.ui.play.PlayScreen

@Composable
fun MainNavigation() {
  val backStack = rememberNavBackStack(Home)

  NavDisplay(
    backStack = backStack,
    onBack = { backStack.removeLastOrNull() },
    entryProvider =
      entryProvider {
        entry<Home> {
          HomeScreen(
              serverUrl = "10.0.2.2:8000",
              onPlay = { layoutId -> backStack.add(Play(layoutId)) },
              onEdit = { layoutId -> backStack.add(Play(layoutId, isEditMode = true)) }
          )
        }
        entry<Play> { play ->
          PlayScreen(
              serverUrl = "10.0.2.2:8000",
              layoutId = play.layoutId,
              isEditMode = play.isEditMode,
              onExit = { backStack.removeLastOrNull() }
          )
        }
      },
  )
}
