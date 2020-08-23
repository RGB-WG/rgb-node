package org.lnpbp.rgbnode.model;

import com.fasterxml.jackson.databind.PropertyNamingStrategy;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

import java.util.List;

@JsonNaming(PropertyNamingStrategy.SnakeCaseStrategy.class)
public class TransferArgs {
    private final List<String> inputs;
    private final List<IssueArgs.CoinAllocation> allocate;
    private final String invoice;
    private final String prototype_psbt;
    private final Integer fee;
    private final String change;
    private final String consignment_file;
    private final String transaction_file;

    public TransferArgs(List<String> inputs, List<IssueArgs.CoinAllocation> allocate, String invoice, String prototype_psbt, Integer fee, String change, String consignment_file, String transaction_file) {
        this.inputs = inputs;
        this.allocate = allocate;
        this.invoice = invoice;
        this.prototype_psbt = prototype_psbt;
        this.fee = fee;
        this.change = change;
        this.consignment_file = consignment_file;
        this.transaction_file = transaction_file;
    }

    public List<String> getInputs() {
        return inputs;
    }

    public List<IssueArgs.CoinAllocation> getAllocate() {
        return allocate;
    }

    public String getInvoice() {
        return invoice;
    }

    public String getPrototype_psbt() {
        return prototype_psbt;
    }

    public Integer getFee() {
        return fee;
    }

    public String getChange() {
        return change;
    }

    public String getConsignment_file() {
        return consignment_file;
    }

    public String getTransaction_file() {
        return transaction_file;
    }
}
