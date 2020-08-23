package org.lnpbp.rgbnode.model;

import com.fasterxml.jackson.databind.PropertyNamingStrategy;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

import java.util.HashMap;

@JsonNaming(PropertyNamingStrategy.SnakeCaseStrategy.class)
public class StartRgbArgs {
    private final String network;
    private final String stashEndpoint;
    private final HashMap<String, String> contractEndpoints;
    private final boolean threaded;
    private final String datadir;

    public StartRgbArgs(String network, String stashEndpoint, HashMap<String, String> contractEndpoints, boolean threaded, String datadir) {
        this.network = network;
        this.stashEndpoint = stashEndpoint;
        this.contractEndpoints = contractEndpoints;
        this.threaded = threaded;
        this.datadir = datadir;
    }

    public String getNetwork() {
        return network;
    }

    public String getStashEndpoint() {
        return stashEndpoint;
    }

    public HashMap<String, String> getContractEndpoints() {
        return contractEndpoints;
    }

    public boolean isThreaded() {
        return threaded;
    }

    public String getDatadir() {
        return datadir;
    }
}
