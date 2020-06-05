package org.lnpbp.demoapp;

import android.app.Application;
import android.util.Log;

import org.lnpbp.rgbnode.COpaqueStruct;
import org.lnpbp.rgbnode.rgb_node;

public class DemoApp extends Application {
    public COpaqueStruct runtime;

    @Override
    public void onCreate() {
        super.onCreate();

        Log.i("RGB_NODE", "loading library");
        System.loadLibrary("rgb_node");

        final String datadir = getFilesDir().toString();
        String network = "testnet";
        this.runtime = rgb_node.start_rgb("{\"network\":\"" + network + "\", \"stash_endpoint\":\"ipc:" + datadir + "/" + network + "/stashd.rpc\", \"contract_endpoints\":{\"Fungible\":\"ipc:" + datadir + "/" + network + "/fungibled.rpc\"}, \"threaded\": true, \"datadir\":\"" + datadir + "\"}");
    }
}
