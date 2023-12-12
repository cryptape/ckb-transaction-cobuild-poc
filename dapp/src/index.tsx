import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom";
import { Script, OutPoint } from "@ckb-lumos/lumos";
import { tmAccounts, Wallet } from "./tmWallet";
import { capacityOf, generateAccountFromPrivateKey, querySporeCells, transfer, createBuildingPacket, giveMessage, getTxStatus } from "./lib";

const app = document.getElementById("root");
ReactDOM.render(<App />, app);

export function App() {

  const [echoMessage, setEchoMessage] = useState("");
  const [buildingPacket, setBuildingPacket] = useState(new Uint8Array());
  const [walletFrom, setWalletFrom] = useState<Wallet>(null);
  const [aliceSporeList, setAliceSporeList] = useState([]);
  const [bobSporeList, setBobSporeList] = useState([]);
  const [showButton, setShowButton] = useState(false);
  const [txHash, setTxHash] = useState("");

  const [count, setCount] = useState(0);

  useEffect(() => {

    async function fetchAliceSporeList() {
      try {
        const data = await querySporeCells(tmAccounts.alice.lock);
        setAliceSporeList(data)
      } catch (error) {
        console.log(error)
      }
    }

    async function fetchBobSporeList() {
      try {
        const data = await querySporeCells(tmAccounts.bob.lock);
        setBobSporeList(data)
      } catch (error) {
        console.log(error)
      }
    }

    async function updateTxStatus() {
      if (txHash != "") {
        let status = await getTxStatus(txHash)
        setEchoMessage(txHash + ": " + status)
        if (status === "committed") {
          setTxHash("")
        }
      }
    }

    const timer = setInterval(() => {
      console.log(txHash)
      fetchAliceSporeList();
      fetchBobSporeList();
      updateTxStatus();
    }, 2000);
    return () => clearInterval(timer);

  }, [txHash]);


  const aliceSpores = aliceSporeList.map(spore =>
    <li key={spore.sporeID}>
      <p>Spore ID: {spore.sporeID}</p>
      <img src={spore.b64Data} width="64" height="64"></img>
      <button onClick={() => { onTransfer(tmAccounts.alice, tmAccounts.bob.lock, spore.outPoint) }}>Transfer to Bob</button>
    </li>
  );
  const bobSpores = bobSporeList.map(spore =>
    <li key={spore.sporeID}>
      <p>Spore ID: {spore.sporeID}</p>
      <img src={spore.b64Data} width="64" height="64"></img>
      <button onClick={() => { onTransfer(tmAccounts.bob, tmAccounts.alice.lock, spore.outPoint) }}>Transfer to Alice</button>
    </li>
  );

  async function onTransfer(from: Wallet, to: Script, outPoint: OutPoint) {
    setWalletFrom(from)
    let bp = await createBuildingPacket(to, outPoint);
    let alertMessage = giveMessage(bp);
    setEchoMessage(alertMessage);
    setBuildingPacket(bp);
    setShowButton(true);
  }

  async function onConfirm() {
    let hash = await walletFrom.signAndSendBuildingPacket(buildingPacket)
    setTxHash(hash)
    setShowButton(false)
  }

  async function onCancel() {
    setEchoMessage("")
    setShowButton(false)
  }

  return (
    <div>
      <h2 style={{ color: "#D1BA74" }}>Spore Transaction Cobuild Demo</h2>
      <code style={{ whiteSpace: 'pre-wrap' }}>{echoMessage}</code>
      <button style={{ display: showButton ? "block" : "none" }} onClick={() => { onConfirm() }}>签名发送</button>
      <button style={{ display: showButton ? "block" : "none" }} onClick={() => { onCancel() }}>取消</button>
      <p style={{ color: "#19CAAD" }}>Alice: {tmAccounts.alice.address}</p>
      <ul style={{ color: "#19CAAD" }}>{aliceSpores}</ul>
      <p style={{ color: "#F4606C" }}>Bob: {tmAccounts.bob.address}</p>
      <ul style={{ color: "#F4606C" }}>{bobSpores}</ul>
    </div>
  );
}
