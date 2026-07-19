package com.example.virtualgamepad.ui.home

import android.widget.Toast
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.example.virtualgamepad.data.GamepadLayout
import com.example.virtualgamepad.data.LayoutManager
import com.example.virtualgamepad.data.NetworkManager
import java.util.UUID

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(serverUrl: String, onPlay: (String) -> Unit, onEdit: (String) -> Unit) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboardManager.current
    
    val connectionStatus = NetworkManager.connectionStatus
    val isConnected = NetworkManager.isConnected
    val isApproved = NetworkManager.isApproved

    var connectionType by remember { mutableStateOf("WiFi") }
    var expanded by remember { mutableStateOf(false) }
    
    var showConnectDialog by remember { mutableStateOf(false) }
    var inputUrl by remember { mutableStateOf(serverUrl) }
    var urlDropdownExpanded by remember { mutableStateOf(false) }

    // Layouts State
    var refreshTrigger by remember { mutableStateOf(0) }
    val layouts = remember(refreshTrigger) { LayoutManager.getAllLayouts() }

    // FAB State
    var isFabExpanded by remember { mutableStateOf(false) }
    val fabRotation by animateFloatAsState(targetValue = if (isFabExpanded) 45f else 0f, label = "fabRotate")
    
    // Import Dialog State
    var showImportDialog by remember { mutableStateOf(false) }
    var importText by remember { mutableStateOf("") }

    // Auto-connect on launch
    LaunchedEffect(Unit) {
        if (!isConnected) {
            val saved = NetworkManager.getSavedUrls()
            val last = saved.lastOrNull()
            if (last != null && last.isNotBlank()) {
                inputUrl = last
                NetworkManager.connect(last)
            }
        }
    }

    if (showConnectDialog) {
        val savedUrls = NetworkManager.getSavedUrls()
        AlertDialog(
            onDismissRequest = { showConnectDialog = false },
            title = { Text("Connect to Server") },
            text = {
                Column {
                    ExposedDropdownMenuBox(
                        expanded = urlDropdownExpanded,
                        onExpandedChange = { urlDropdownExpanded = it }
                    ) {
                        OutlinedTextField(
                            value = inputUrl,
                            onValueChange = { inputUrl = it },
                            label = { Text("Server IP/URL") },
                            modifier = Modifier.menuAnchor().fillMaxWidth(),
                            singleLine = true,
                            placeholder = { Text("e.g. 192.168.1.5:8000") }
                        )
                        if (savedUrls.isNotEmpty()) {
                            ExposedDropdownMenu(
                                expanded = urlDropdownExpanded,
                                onDismissRequest = { urlDropdownExpanded = false }
                            ) {
                                savedUrls.forEach { url ->
                                    DropdownMenuItem(
                                        text = { Text(url) },
                                        onClick = {
                                            inputUrl = url
                                            urlDropdownExpanded = false
                                        }
                                    )
                                }
                            }
                        }
                    }
                }
            },
            confirmButton = {
                Button(onClick = {
                    if (inputUrl.isNotBlank()) {
                        NetworkManager.saveUrl(inputUrl)
                        NetworkManager.connect(inputUrl)
                    }
                    showConnectDialog = false
                }) {
                    Text("Connect")
                }
            },
            dismissButton = {
                TextButton(onClick = { showConnectDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    if (showImportDialog) {
        AlertDialog(
            onDismissRequest = { showImportDialog = false },
            title = { Text("Import Layout") },
            text = {
                OutlinedTextField(
                    value = importText,
                    onValueChange = { importText = it },
                    label = { Text("Paste Layout Code") },
                    modifier = Modifier.fillMaxWidth()
                )
            },
            confirmButton = {
                Button(onClick = {
                    if (importText.isNotBlank()) {
                        val imported = LayoutManager.importLayout(importText)
                        if (imported != null) {
                            Toast.makeText(context, "Layout Imported!", Toast.LENGTH_SHORT).show()
                            refreshTrigger++
                            showImportDialog = false
                            importText = ""
                        } else {
                            Toast.makeText(context, "Invalid Code", Toast.LENGTH_SHORT).show()
                        }
                    }
                }) {
                    Text("Import")
                }
            },
            dismissButton = {
                TextButton(onClick = { showImportDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    Scaffold(
        topBar = {
            Surface(
                modifier = Modifier.statusBarsPadding(),
                color = MaterialTheme.colorScheme.surface,
                shadowElevation = 4.dp
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box {
                        TextButton(
                            onClick = { expanded = true },
                            contentPadding = PaddingValues(0.dp)
                        ) {
                            Text(
                                text = "$connectionType ▼",
                                color = MaterialTheme.colorScheme.primary,
                                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold)
                            )
                        }
                        DropdownMenu(
                            expanded = expanded,
                            onDismissRequest = { expanded = false }
                        ) {
                            DropdownMenuItem(
                                text = { Text("WiFi") },
                                onClick = { connectionType = "WiFi"; expanded = false }
                            )
                            DropdownMenuItem(
                                text = { Text("USB (lower latency)") },
                                onClick = { connectionType = "USB"; expanded = false }
                            )
                        }
                    }

                    Column(horizontalAlignment = Alignment.End) {
                        Button(
                            onClick = {
                                if (isConnected) {
                                    NetworkManager.disconnect()
                                } else {
                                    showConnectDialog = true
                                }
                            },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = if (isConnected) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
                            )
                        ) {
                            Text(if (isConnected) "Disconnect" else "Connect")
                        }
                        Text(
                            text = connectionStatus,
                            style = MaterialTheme.typography.bodySmall.copy(
                                color = if (isApproved) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                            ),
                            modifier = Modifier.padding(top = 4.dp)
                        )
                    }
                }
            }
        },
        floatingActionButton = {
            Column(horizontalAlignment = Alignment.End) {
                AnimatedVisibility(
                    visible = isFabExpanded,
                    enter = fadeIn() + expandVertically(),
                    exit = fadeOut() + shrinkVertically()
                ) {
                    Column(
                        horizontalAlignment = Alignment.End,
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                        modifier = Modifier.padding(bottom = 16.dp)
                    ) {
                        ExtendedFloatingActionButton(
                            onClick = {
                                isFabExpanded = false
                                showImportDialog = true
                            },
                            containerColor = MaterialTheme.colorScheme.secondaryContainer,
                            contentColor = MaterialTheme.colorScheme.onSecondaryContainer
                        ) {
                            Text("Import Layout")
                        }
                        ExtendedFloatingActionButton(
                            onClick = {
                                val id = UUID.randomUUID().toString()
                                LayoutManager.saveLayout(LayoutManager.createDefaultLayout(id))
                                isFabExpanded = false
                                refreshTrigger++
                            },
                            containerColor = MaterialTheme.colorScheme.surfaceVariant,
                            contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                        ) {
                            Text("New Default")
                        }
                        ExtendedFloatingActionButton(
                            onClick = {
                                val id = UUID.randomUUID().toString()
                                LayoutManager.saveLayout(LayoutManager.createRacingLayout(id))
                                isFabExpanded = false
                                refreshTrigger++
                            },
                            containerColor = MaterialTheme.colorScheme.surfaceVariant,
                            contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                        ) {
                            Text("New Racing")
                        }
                        ExtendedFloatingActionButton(
                            onClick = {
                                val id = UUID.randomUUID().toString()
                                LayoutManager.saveLayout(LayoutManager.createFlightLayout(id))
                                isFabExpanded = false
                                refreshTrigger++
                            },
                            containerColor = MaterialTheme.colorScheme.surfaceVariant,
                            contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                        ) {
                            Text("New Flight Pad")
                        }
                    }
                }
                FloatingActionButton(
                    onClick = { isFabExpanded = !isFabExpanded },
                    containerColor = MaterialTheme.colorScheme.primary
                ) {
                    Text(
                        "+",
                        modifier = Modifier.rotate(fabRotation),
                        style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold)
                    )
                }
            }
        },
        containerColor = MaterialTheme.colorScheme.background
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                Text(
                    text = "My Layouts",
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                    color = MaterialTheme.colorScheme.onBackground,
                    modifier = Modifier.padding(bottom = 8.dp)
                )
            }
            items(layouts) { layout ->
                LayoutCard(
                    layout = layout,
                    onPlay = { onPlay(layout.id) },
                    onEdit = { onEdit(layout.id) },
                    onDuplicate = {
                        LayoutManager.duplicateLayout(layout)
                        refreshTrigger++
                    },
                    onExport = {
                        val exported = LayoutManager.exportLayout(layout)
                        clipboardManager.setText(AnnotatedString(exported))
                        Toast.makeText(context, "Layout Code Copied!", Toast.LENGTH_SHORT).show()
                    },
                    onDelete = {
                        LayoutManager.deleteLayout(layout.id)
                        refreshTrigger++
                    }
                )
            }
        }
    }
}

@Composable
fun LayoutCard(
    layout: GamepadLayout, 
    onPlay: () -> Unit, 
    onEdit: () -> Unit, 
    onDuplicate: () -> Unit, 
    onExport: () -> Unit, 
    onDelete: () -> Unit
) {
    var showMenu by remember { mutableStateOf(false) }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .height(100.dp)
            .clip(RoundedCornerShape(16.dp))
            .clickable { onPlay() },
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column {
                Text(
                    text = layout.name,
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.SemiBold),
                    color = MaterialTheme.colorScheme.onSurface
                )
                Text(
                    text = "Tap to play",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f)
                )
            }
            
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconButton(onClick = onEdit) {
                    Text(
                        "Edit",
                        color = MaterialTheme.colorScheme.primary,
                        style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold)
                    )
                }
                Box {
                    IconButton(onClick = { showMenu = true }) {
                        Text(
                            "⋮",
                            color = MaterialTheme.colorScheme.onSurface,
                            style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold)
                        )
                    }
                    DropdownMenu(
                        expanded = showMenu,
                        onDismissRequest = { showMenu = false }
                    ) {
                        DropdownMenuItem(
                            text = { Text("Duplicate") },
                            onClick = { showMenu = false; onDuplicate() }
                        )
                        DropdownMenuItem(
                            text = { Text("Copy / Export") },
                            onClick = { showMenu = false; onExport() }
                        )
                        DropdownMenuItem(
                            text = { Text("Delete", color = MaterialTheme.colorScheme.error) },
                            onClick = { showMenu = false; onDelete() }
                        )
                    }
                }
            }
        }
    }
}
