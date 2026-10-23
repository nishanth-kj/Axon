package com.example.axon_android;

/**
 * Loads the Rust node library and exposes start, stop, and log calls.
 * The matching symbols live in {@code axon-android/rust}.
 */
public final class NodeBridge {
    public static final boolean LOADED;

    static {
        boolean loaded = false;
        try {
            System.loadLibrary("axon_android");
            loaded = true;
        } catch (UnsatisfiedLinkError ignored) {
            loaded = false;
        }
        LOADED = loaded;
    }

    private NodeBridge() {}

    public static native String nativeStart(
            String registry,
            String secret,
            int verifyPort,
            int relayPort,
            int socksPort,
            boolean useTor);

    public static native void nativeStop();

    public static native String nativeStatus();

    public static native String nativeNotice();

    public static native String nativeLogs();
}
