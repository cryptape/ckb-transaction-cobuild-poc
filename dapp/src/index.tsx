import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom";
import { Script } from "@ckb-lumos/lumos";
import { tmAccounts } from "./tmWallet";
import { capacityOf, generateAccountFromPrivateKey, querySporeCells, transfer, createBuildingPacket, giveMessage } from "./lib";

const app = document.getElementById("root");
ReactDOM.render(<App />, app);

export function App() {

  const [echoMessage, setEchoMessage] = useState("");
  const [buildingPacket, setBuildingPacket] = useState(new Uint8Array());
  const [aliceSporeList, setAliceSporeList] = useState([]);
  const [bobSporeList, setBobSporeList] = useState([]);

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

    const timer = setInterval(() => {
      fetchAliceSporeList();
      fetchBobSporeList();
    }, 1000);
    return () => clearInterval(timer);

  }, []);


  const aliceSpores = aliceSporeList.map(spore =>
    <li key={spore.sporeID}>
      <p>Spore ID: {spore.sporeID}</p>
      <img src={spore.b64Data} width="64" height="64"></img>
      <button onClick={() => { transfer(tmAccounts.alice, tmAccounts.bob.lock, spore.outPoint) }}>Transfer to Bob</button>
    </li>
  );
  const bobSpores = bobSporeList.map(spore =>
    <li key={spore.sporeID}>
      <p>Spore ID: {spore.sporeID}</p>
      <img src={spore.b64Data} width="64" height="64"></img>
      <button onClick={() => { transfer(tmAccounts.bob, tmAccounts.alice.lock, spore.outPoint) }}>Transfer to Alice</button>
    </li>
  );

  return (
    <div>
      <h2 style={{color: "#D1BA74"}}>Spore Transaction Cobuild Demo</h2>
      <p>{ echoMessage }</p>
      <p style={{color: "#19CAAD"}}>Alice: {tmAccounts.alice.address}</p>
      <ul style={{color: "#19CAAD"}}>{aliceSpores}</ul>
      <p style={{color: "#F4606C"}}>Bob: {tmAccounts.bob.address}</p>
      <ul style={{color: "#F4606C"}}>{bobSpores}</ul>
    </div>
  );
}
