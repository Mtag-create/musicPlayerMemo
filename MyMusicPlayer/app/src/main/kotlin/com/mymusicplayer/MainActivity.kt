package com.mymusicplayer

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.Composable
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.mymusicplayer.ui.navigation.Screen
import com.mymusicplayer.ui.screen.PlayerScreen
import com.mymusicplayer.ui.screen.SettingsScreen
import com.mymusicplayer.ui.screen.HistoryScreen
import com.mymusicplayer.ui.screen.TrackListScreen
import com.mymusicplayer.ui.theme.MyMusicPlayerTheme
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.compose.material3.MaterialTheme
import androidx.lifecycle.viewmodel.compose.viewModel
import com.mymusicplayer.player.PlayerViewModel
import com.mymusicplayer.ui.screen.EditTracksScreen

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MyMusicPlayerTheme {
                MyMusicPlayerApp()
            }
        }
    }
}

@Composable
private fun MyMusicPlayerApp() {
    val navController: NavHostController = rememberNavController()
    val playerViewModel: PlayerViewModel = viewModel()

    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background
    ) {
        NavHost(navController = navController, startDestination = Screen.Player.route) {
            composable(Screen.Player.route) {
                PlayerScreen(
                    onNavigateToSettings = { navController.navigate(Screen.Settings.route) },
                    onNavigateToHistory = { navController.navigate(Screen.History.route) },
                    onNavigateToTrackList = { navController.navigate(Screen.TrackList.route) },
                    viewModel = playerViewModel
                )
            }
            composable(Screen.Settings.route) {
                SettingsScreen(
                    onNavigateBack = { navController.popBackStack() },
                    onNavigateToEditTracks = { navController.navigate(Screen.EditTracks.route) },
                    viewModel = playerViewModel
                )
            }
            composable(Screen.EditTracks.route) {
                EditTracksScreen(
                    onNavigateBack = { navController.popBackStack() },
                    viewModel = playerViewModel
                )
            }
            composable(Screen.History.route) {
                HistoryScreen(
                    onNavigateBack = { navController.popBackStack() },
                    viewModel = playerViewModel
                )
            }
            composable(Screen.TrackList.route) {
                TrackListScreen(
                    onNavigateBack = { navController.popBackStack() },
                    viewModel = playerViewModel
                )
            }
        }
    }
}