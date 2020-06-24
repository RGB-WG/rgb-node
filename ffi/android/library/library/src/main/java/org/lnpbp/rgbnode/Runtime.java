package org.lnpbp.rgbnode;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;

import org.lnpbp.rgbnode.model.IssueArgs;
import org.lnpbp.rgbnode.model.StartRgbArgs;
import org.lnpbp.rgbnode.model.TransferArgs;
import org.lnpbp.rgbnode_autogen.COpaqueStruct;
import org.lnpbp.rgbnode_autogen.rgb_node;

import java.util.HashMap;
import java.util.List;

public class Runtime {
    private final COpaqueStruct runtime;
    private final ObjectMapper mapper;

    public Runtime(final String network, final String stashEndpoint, final HashMap<String, String> contractEndpoints, final boolean threaded, final String datadir) throws RuntimeException {
        mapper = new ObjectMapper();

        final StartRgbArgs args = new StartRgbArgs(network, stashEndpoint, contractEndpoints, threaded, datadir);
        try {
            final String jsonArgs = mapper.writeValueAsString(args);
            this.runtime = rgb_node.start_rgb(jsonArgs);
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }
    }

    public void issue(final String network, final String ticker, final String name, final String description, final String issueStructure, final List<IssueArgs.CoinAllocation> allocations, final Integer precision, final List<IssueArgs.SealSpec> pruneSeals, final Integer dustLimit) throws RuntimeException {
        final IssueArgs args = new IssueArgs(network, ticker, name, description, issueStructure, allocations, precision, pruneSeals, dustLimit);
        try {
            final String jsonArgs = mapper.writeValueAsString(args);
            rgb_node.issue(this.runtime, jsonArgs);
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }
    }

    public void transfer(List<String> inputs, List<IssueArgs.CoinAllocation> allocate, String invoice, String prototype_psbt, Integer fee, String change, String consignment_file, String transaction_file) throws RuntimeException {
        final TransferArgs args = new TransferArgs(inputs, allocate, invoice, prototype_psbt, fee, change, consignment_file, transaction_file);
        try {
            final String jsonArgs = mapper.writeValueAsString(args);
            rgb_node.transfer(this.runtime, jsonArgs);
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }
    }
}
