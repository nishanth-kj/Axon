package com.example.axon_android;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.Service;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;

/**
 * Keeps the process alive while the Rust node is running.
 * Settings come from the same preferences the activity edits.
 */
public class NodeService extends Service {
    static final String PREFS = "axon";
    static volatile String lastResult = "";

    private static final String CHANNEL = "axon-node";
    private static final int NOTIFICATION_ID = 1;

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        promoteToForeground();
        if (!NodeBridge.LOADED) {
            lastResult = "Native library missing.";
            stopForeground(STOP_FOREGROUND_REMOVE);
            stopSelf();
            return START_NOT_STICKY;
        }

        SharedPreferences prefs = getSharedPreferences(PREFS, MODE_PRIVATE);
        String result = NodeBridge.nativeStart(
                prefs.getString("registry", "http://127.0.0.1:7400"),
                prefs.getString("secret", "Nishanth"),
                parsePort(prefs.getString("verify", "8080")),
                parsePort(prefs.getString("relay", "8081")),
                parsePort(prefs.getString("socks", "1080")),
                prefs.getBoolean("tor", false));
        lastResult = result;
        if (!"ok".equals(result) && !"The node is already running.".equals(result)) {
            stopForeground(STOP_FOREGROUND_REMOVE);
            stopSelf();
            return START_NOT_STICKY;
        }
        return START_STICKY;
    }

    @Override
    public void onDestroy() {
        if (NodeBridge.LOADED) {
            NodeBridge.nativeStop();
        }
        super.onDestroy();
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    private void promoteToForeground() {
        NotificationManager manager = getSystemService(NotificationManager.class);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && manager != null) {
            NotificationChannel channel = new NotificationChannel(
                    CHANNEL, "Axon node", NotificationManager.IMPORTANCE_LOW);
            channel.setDescription("Shown while the local proxy is running.");
            manager.createNotificationChannel(channel);
        }

        Notification.Builder builder = Build.VERSION.SDK_INT >= Build.VERSION_CODES.O
                ? new Notification.Builder(this, CHANNEL)
                : new Notification.Builder(this);
        Notification notification = builder
                .setContentTitle("Axon")
                .setContentText("Local proxy is running")
                .setSmallIcon(android.R.drawable.stat_notify_sync)
                .setOngoing(true)
                .build();

        if (Build.VERSION.SDK_INT >= 34) {
            startForeground(
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE);
        } else {
            startForeground(NOTIFICATION_ID, notification);
        }
    }

    private static int parsePort(String value) {
        try {
            return Integer.parseInt(value.trim());
        } catch (NumberFormatException error) {
            return 0;
        }
    }
}
