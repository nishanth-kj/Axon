package com.example.axon_android;

import android.Manifest;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.View;

import androidx.appcompat.app.AppCompatActivity;
import androidx.core.content.ContextCompat;

import com.example.axon_android.databinding.ActivityMainBinding;

public class MainActivity extends AppCompatActivity {
    private ActivityMainBinding binding;
    private final Handler handler = new Handler(Looper.getMainLooper());
    private final Runnable poll = new Runnable() {
        @Override
        public void run() {
            refresh();
            handler.postDelayed(this, 1000);
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        binding = ActivityMainBinding.inflate(getLayoutInflater());
        setContentView(binding.getRoot());
        loadPrefs();

        binding.startButton.setOnClickListener(v -> startNode());
        binding.stopButton.setOnClickListener(v -> stopService(new Intent(this, NodeService.class)));

        if (Build.VERSION.SDK_INT >= 33
                && ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
                != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS}, 1);
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        refresh();
        handler.postDelayed(poll, 1000);
    }

    @Override
    protected void onPause() {
        handler.removeCallbacks(poll);
        savePrefs();
        super.onPause();
    }

    private void startNode() {
        savePrefs();
        if (!NodeBridge.LOADED) {
            showNotice("The native node library is not in this build. Package libaxon_android.so and rebuild the app.");
            return;
        }
        Intent intent = new Intent(this, NodeService.class);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent);
        } else {
            startService(intent);
        }
        refresh();
    }

    private void refresh() {
        if (!NodeBridge.LOADED) {
            binding.statusText.setText("Library missing");
            binding.socksText.setText(socksAddress());
            showNotice("The native node library is not in this build. Package libaxon_android.so and rebuild the app.");
            setEditing(true);
            binding.startButton.setEnabled(true);
            binding.stopButton.setVisibility(View.GONE);
            return;
        }

        String status = NodeBridge.nativeStatus();
        String notice = NodeBridge.nativeNotice();
        if (notice == null || notice.isEmpty()) {
            notice = NodeService.lastResult;
            if ("ok".equals(notice) || "The node is already running.".equals(notice)) {
                notice = "";
            }
        }
        showNotice(notice);
        binding.logText.setText(NodeBridge.nativeLogs());
        binding.socksText.setText(socksAddress());

        boolean stopped = "stopped".equals(status);
        boolean stopping = "stopping".equals(status);
        binding.statusText.setText(stopped ? "Stopped" : stopping ? "Stopping" : "Running");
        setEditing(stopped);
        binding.startButton.setVisibility(stopped ? View.VISIBLE : View.GONE);
        binding.stopButton.setVisibility(stopped ? View.GONE : View.VISIBLE);
        binding.stopButton.setEnabled(!stopping);
        binding.stopButton.setText(stopping ? "Stopping" : "Stop node");
    }

    private void setEditing(boolean editable) {
        binding.registryInput.setEnabled(editable);
        binding.secretInput.setEnabled(editable);
        binding.verifyInput.setEnabled(editable);
        binding.relayInput.setEnabled(editable);
        binding.socksInput.setEnabled(editable);
        binding.torSwitch.setEnabled(editable);
    }

    private void showNotice(String notice) {
        boolean visible = notice != null && !notice.isEmpty();
        binding.noticeText.setVisibility(visible ? View.VISIBLE : View.GONE);
        binding.noticeText.setText(visible ? notice : "");
    }

    private String socksAddress() {
        return "127.0.0.1:" + binding.socksInput.getText().toString().trim();
    }

    private SharedPreferences prefs() {
        return getSharedPreferences(NodeService.PREFS, MODE_PRIVATE);
    }

    private void loadPrefs() {
        SharedPreferences prefs = prefs();
        binding.registryInput.setText(prefs.getString("registry", "http://127.0.0.1:7400"));
        binding.secretInput.setText(prefs.getString("secret", "Nishanth"));
        binding.verifyInput.setText(prefs.getString("verify", "8080"));
        binding.relayInput.setText(prefs.getString("relay", "8081"));
        binding.socksInput.setText(prefs.getString("socks", "1080"));
        binding.torSwitch.setChecked(prefs.getBoolean("tor", false));
    }

    private void savePrefs() {
        prefs().edit()
                .putString("registry", text(binding.registryInput))
                .putString("secret", text(binding.secretInput))
                .putString("verify", text(binding.verifyInput))
                .putString("relay", text(binding.relayInput))
                .putString("socks", text(binding.socksInput))
                .putBoolean("tor", binding.torSwitch.isChecked())
                .apply();
    }

    private static String text(android.widget.EditText field) {
        return field.getText().toString().trim();
    }
}
