package com.example.virtualgamepad

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable data object Home : NavKey
@Serializable data class Play(val layoutId: String, val isEditMode: Boolean = false) : NavKey
