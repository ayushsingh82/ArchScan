"use client"
import React, { useState, useEffect } from "react";
import { SigningArchwayClient } from "@archwayhq/arch3.js";
import BigNumber from "bignumber.js";
import { Button } from "@/components/ui/button";

const REGISTRY_CONTRACT = "archway1275jwjpktae4y4y0cdq274a2m0jnpekhttnfuljm6n59wnpyd62qppqxq0";

const Blockchain = {
  chainId: "constantine-3",
  chainName: "Constantine",
  rpc: "https://rpc.constantine.archway.io",
  stakeCurrency: { coinDenom: "ARCH", coinMinimalDenom: "aarch", coinDecimals: 6 },
  bech32Config: {
    bech32PrefixAccAddr: "archway",
    bech32PrefixAccPub: "archwaypub",
    bech32PrefixValAddr: "archwayvaloper",
    bech32PrefixValPub: "archwayvaloperpub",
    bech32PrefixConsAddr: "archwayvalcons",
    bech32PrefixConsPub: "archwayvalconspub",
  },
  currencies: [{ coinDenom: "ARCH", coinMinimalDenom: "aarch", coinDecimals: 18 }],
  feeCurrencies: [
    {
      coinDenom: "ARCH",
      coinMinimalDenom: "aarch",
      coinDecimals: 18
    },
  ],
  features: ["cosmwasm"],
};

const TestnetBlockchain = {
  chainId: "constantine-3",
  chainName: "Archway Testnet",
  rpc: "https://rpc.constantine.archway.io",
  stakeCurrency: { coinDenom: "ARCH", coinMinimalDenom: "aarch", coinDecimals: 6 },
  bech32Config: {
    bech32PrefixAccAddr: "archway",
    bech32PrefixAccPub: "archwaypub",
    bech32PrefixValAddr: "archwayvaloper",
    bech32PrefixValPub: "archwayvaloperpub",
    bech32PrefixConsAddr: "archwayvalcons",
    bech32PrefixConsPub: "archwayvalconspub",
  },
  currencies: [{ coinDenom: "ARCH", coinMinimalDenom: "aarch", coinDecimals: 18 }],
  feeCurrencies: [
    {
      coinDenom: "ARCH",
      coinMinimalDenom: "aarch",
      coinDecimals: 18,
      gasPriceStep: { low: 0, average: 0.1, high: 0.2 },
    },
  ],
  features: ["cosmwasm"],
  blockExplorer: "https://www.mintscan.io/archway-testnet",
};


async function getClient() {
  try {
    await window.keplr.experimentalSuggestChain(TestnetBlockchain);
    await window.keplr.enable(TestnetBlockchain.chainId);
    window.keplr.defaultOptions = { sign: { preferNoSetFee: true } };
    const signer = await window.getOfflineSignerAuto(TestnetBlockchain.chainId);
    const client = await SigningArchwayClient.connectWithSigner(TestnetBlockchain.rpc, signer);
    return client;
  } catch (error) {
    throw new Error("Failed to initialize Keplr or get client: " + error.message);
  }
}

async function resolveRecord(name) {
  if (!name) return { error: "Name is required." };

  try {
    const client = await getClient();
    const entrypoint = {
      resolve_record: {
        name: name,
      },
    };
    const query = await client.queryClient.wasm.queryContractSmart(REGISTRY_CONTRACT, entrypoint);
    return {
      address: query.address,
      resolvedRecord: query,
      error: null,
    };
  } catch (error) {
    console.error("Error resolving record:", error);
    return {
      error: error.message || "Error resolving record.",
    };
  }
}

function ProfilePage() {
  const [name, setName] = useState("");
  const [resolvedRecord, setResolvedRecord] = useState(null);
  const [amount, setAmount] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    const initKeplr = async () => {
      if (!window.getOfflineSignerAuto || !window.keplr) {
        alert("Please install keplr extension");
      } else {
        if (window.keplr.experimentalSuggestChain) {
          try {
            await window.keplr.experimentalSuggestChain(TestnetBlockchain);
            window.keplr.defaultOptions = { sign: { preferNoSetFee: true } };
          } catch {
            alert("Failed to suggest the chain");
          }
        } else {
          alert("Please use the recent version of keplr extension");
        }
      }
    };

    initKeplr();
  }, []);

  const handleSearch = async () => {
    const result = await resolveRecord(name);
    setResolvedRecord(result);
    setMessage("");
  };

  return (
    <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <div className="rounded-lg shadow-lg p-6">
        <h2 className="text-2xl font-bold mb-4">Search Wallet Address</h2>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Enter ID (e.g., kamal.arch)"
          className="border p-2 rounded w-full mb-4"
        />
        <button
          onClick={handleSearch}
          className="bg-blue-500 hover:bg-blue-600 font-bold py-2 px-4 rounded transition duration-300"
        >
          Search
        </button>
        {resolvedRecord && (
          <div className="mt-6">
            {resolvedRecord.error ? (
              <p className="text-red-500">Error: {resolvedRecord.error}</p>
            ) : (
              <>
                <p className="text-gray-500 dark:text-gray-400">Address: {resolvedRecord.address}</p>
                <input
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="Enter amount"
                  className="border p-2 rounded w-full mb-4"
                />
                <button
                  onClick={handlePayDirectly}
                  className="mt-4 bg-green-500 hover:bg-green-600 font-bold py-2 px-4 rounded transition duration-300"
                >
                  Pay Directly
                </button>
                {message && (
                  <p className="mt-2 text-sm text-gray-500">{message}</p>
                )}
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default ProfilePage;