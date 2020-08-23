package org.lnpbp.rgbnode.model;

import com.fasterxml.jackson.databind.PropertyNamingStrategy;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

import java.util.List;

@JsonNaming(PropertyNamingStrategy.SnakeCaseStrategy.class)
public class IssueArgs {
    private final String network;
    private final String ticker;
    private final String name;
    private final String description;
    private final String issueStructure;
    private final List<CoinAllocation> allocations;
    private final Integer precision;
    private final List<SealSpec> pruneSeals;
    private final Integer dustLimit;

    public IssueArgs(String network, String ticker, String name, String description, String issueStructure, List<CoinAllocation> allocations, Integer precision, List<SealSpec> pruneSeals, Integer dustLimit) {
        this.network = network;
        this.ticker = ticker;
        this.name = name;
        this.description = description;
        this.issueStructure = issueStructure;
        this.allocations = allocations;
        this.precision = precision;
        this.pruneSeals = pruneSeals;
        this.dustLimit = dustLimit;
    }

    public String getNetwork() {
        return network;
    }

    public String getTicker() {
        return ticker;
    }

    public String getName() {
        return name;
    }

    public String getDescription() {
        return description;
    }

    public String getIssueStructure() {
        return issueStructure;
    }

    public List<CoinAllocation> getAllocations() {
        return allocations;
    }

    public Integer getPrecision() {
        return precision;
    }

    public List<SealSpec> getPruneSeals() {
        return pruneSeals;
    }

    public Integer getDustLimit() {
        return dustLimit;
    }

    @JsonNaming(PropertyNamingStrategy.SnakeCaseStrategy.class)
    public static class CoinAllocation {
        private final Long coins;
        private final Integer vout;
        private final String txid;

        public CoinAllocation(Long coins, Integer vout, String txid) {
            this.coins = coins;
            this.vout = vout;
            this.txid = txid;
        }

        public Long getCoins() {
            return coins;
        }

        public Integer getVout() {
            return vout;
        }

        public String getTxid() {
            return txid;
        }
    }

    public static class SealSpec {
        private final Integer vout;
        private final String txid;

        public SealSpec(Integer vout, String txid) {
            this.vout = vout;
            this.txid = txid;
        }

        public Integer getVout() {
            return vout;
        }

        public String getTxid() {
            return txid;
        }
    }
}
