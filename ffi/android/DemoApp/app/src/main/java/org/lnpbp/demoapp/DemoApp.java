package org.lnpbp.demoapp;

import android.app.Application;
import android.util.Log;

import org.lnpbp.rgbnode.Runtime;

import java.util.HashMap;

public class DemoApp extends Application {
    private Runtime runtime;

    @Override
    public void onCreate() {
        super.onCreate();

        Log.i("RGB_NODE", "loading library");
        System.loadLibrary("rgb_node");

        final String datadir = getFilesDir().toString();
        final String network = "testnet";

        final HashMap contractEndpoints = new HashMap();
        contractEndpoints.put("Fungible", String.format("ipc:%s/%s/fungibled.rpc", datadir, network));
        this.runtime = new Runtime(network, String.format("ipc:%s/%s/stashd.rpc", datadir, network), contractEndpoints, true, datadir);
   }

    public Runtime getRuntime() {
        return runtime;
    }
}
