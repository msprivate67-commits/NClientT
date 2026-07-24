package com.nclientt.app

import android.os.Bundle
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Android 15+ enforces edge-to-edge layouts for apps targeting recent
        // SDKs. Inset Tauri's WebView by the system bars and any display cutout
        // so the app toolbar is not rendered underneath the notification bar.
        val content = findViewById<View>(android.R.id.content)
        ViewCompat.setOnApplyWindowInsetsListener(content) { view, windowInsets ->
            val safeInsets = windowInsets.getInsets(
                WindowInsetsCompat.Type.systemBars() or
                    WindowInsetsCompat.Type.displayCutout(),
            )
            view.setPadding(
                safeInsets.left,
                safeInsets.top,
                safeInsets.right,
                safeInsets.bottom,
            )
            windowInsets
        }
        ViewCompat.requestApplyInsets(content)
    }
}
