import React from "react";
import { useCallback } from 'react';

import Slider from "@material-ui/core/Slider/Slider";
import Typography from "@material-ui/core/Typography/Typography";

import {
  Account,
  PublicKey,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';

import {
    useConnection,
    sendDepositSequence,
    sendWithdrawSequence,
    getTokenAccounts,
    getTokenLargestAccounts,
    getMintInfo
} from "../util/connection"

import { useWallet } from "../util/wallet";
import ReactEcharts from 'echarts-for-react';

const acc = "vFj/mjPXxWxMoVxwBpRfHKufaxK0RYy3Gd2rAmKlveF7oiinGDnsXlRSbXieC5x6prka4aQGE8tFRz17zLl38w==";
const treasuryAccount = new Account(Buffer.from(acc, "base64"));

let payerAccount = new Account(Buffer.from("xpCzQo06gWIJtRCXglEMkXUQNQG8UrA8yVGDtA93qOQnLtX3TnG+kZCsmHtanJpFluRL958AbUOR7I2HKK4zlg==", "base64"));
let sysvarClockPubKey = new PublicKey('SysvarC1ock11111111111111111111111111111111');
let sysvarSlotHashesPubKey = new PublicKey('SysvarS1otHashes111111111111111111111111111');
let splTokenProgram = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

let programId = new PublicKey("8Nj5RBeppFvrLzF5t4t5i3i3B2ucx9qVUxp2nc5dVDGt");
let treasuryTokenAccount = new PublicKey("6ME9zXExwYxqGV3XiXGVQwvQS6mq5QCucaVEnF5HyQ71");

function hashCode(str) {
    var hash = 0;
    for (var i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    return hash;
}
function intToRGB(i){
    var c = (i & 0x00FFFFFF)
        .toString(16)
        .toUpperCase();

    return "00000".substring(0, 6 - c.length) + c;
}

function getChartData(data) {
    var legendData = [];
    var colors = [];
    var seriesData = [];
    for (var i = 0; i < data.length; i++) {
        var name;
        if (i >= 9) {
            name = "Others";
            if (i == 9) {
                legendData.push(name);
                seriesData.push({
                    name: name,
                    value: data[i][1],
                });
                colors.push('#' + intToRGB(hashCode(name)));
            } else {
                seriesData[9].value = seriesData[9].value + seriesData[i].value;
            }
        } else {
            name = data[i][0];
            legendData.push(name.substring(0, 8) + "...");
            seriesData.push({
                name: name.substring(0, 8) + "...",
                value: data[i][1],
            });
            colors.push('#' + intToRGB(hashCode(name)));
        }
    }
    return {
        legendData: legendData,
        seriesData: seriesData,
        colors: colors
    };
}

export function FundPage() {

    const connection = useConnection();
    const [chartData, setChartData] = React.useState([]);
    const [invalid, setInvalid] = React.useState(true);
    const [roll_value, setRollValue] = React.useState(51);
    const [chance, setChance] = React.useState(50);
    const [profit, setProfit] = React.useState(0.98);
    const [wager, setWager] = React.useState(1);
    const [wager_count, setWagerCount] = React.useState(0);
    const [refresh, setRefresh] = React.useState(0);
    const [balance, setBalance] = React.useState(0);
    const [tokenSupply, setTokenSupply] = React.useState(0);
    const { publicKey, wallet, connected } = useWallet();
    const [depositAmount, setDepositAmount] = React.useState(1);
    const [withdrawAmount, setWithdrawAmount] = React.useState(1);
    const [userTokenAccount, setUserTokenAccount] = React.useState(0);
    const [userTokenBalance, setUserTokenBalance] = React.useState(0);
    const [fundBalance, setFundBalance] = React.useState(0);
    const [fundBalanceDollar, setFundBalanceDollar] = React.useState(0);
    const [maxProfitAllowed, setMaxProfitAllowed] = React.useState(0);

    const treasuryAccountLink = "https://explorer.solana.com/address/" + (treasuryAccount.publicKey ? treasuryAccount.publicKey.toString() : "") + "?cluster=devnet";
    const treasuryTokenAccountLink = "https://explorer.solana.com/address/" + (treasuryTokenAccount ? treasuryTokenAccount.toString() : "") + "?cluster=devnet";

    const refreshTreasuryBalance = React.useCallback(() => {
        (async () => {
          try {
            const balance = await connection.getBalance(
              treasuryAccount.publicKey,
              "singleGossip"
            );
            setFundBalance(balance/LAMPORTS_PER_SOL);
            setFundBalanceDollar(((balance/LAMPORTS_PER_SOL)*1.8).toFixed(2)); // TODO
            setMaxProfitAllowed((balance/LAMPORTS_PER_SOL)*0.01); // TODO
          } catch (err) {
              console.log(err);
          }
        })();
    }, []);

    const setNewProfit = (newValue, newWager) => {
        // let x = (( (   ((100-(newValue-1))) / (newValue-1)+1))*990/1000)-1;
        let sub_under_number_64 = newValue - 1;
        let num = (100 - sub_under_number_64);
        let tmp = (num / sub_under_number_64) + 1;
        let winning_ratio = (tmp * 990 / 1000) - 1;
        let newProfit = winning_ratio*newWager;
        setProfit(newProfit);
        setInvalid(newProfit > maxProfitAllowed);
    };

    const depositToTreasury = useCallback(() => {
        console.log('depositing to treasury');
        if (connected) {
            (async () => {
                await sendDepositSequence(
                    depositAmount,
                    wallet,
                    connection,
                    programId,
                    payerAccount,
                    splTokenProgram,
                    treasuryAccount,
                    treasuryTokenAccount,
                    userTokenAccount,
                    setUserTokenAccount,
                    setRefresh
                );
            })();
        }
    }, [connected, wallet, depositAmount, payerAccount, userTokenAccount]);

    const withdrawFromTreasury = useCallback(() => {
        console.log('withdrawing from treasury');
        if (connected && userTokenBalance > 0) {
            (async () => {
                await sendWithdrawSequence(
                    withdrawAmount,
                    wallet,
                    connection,
                    programId,
                    payerAccount,
                    splTokenProgram,
                    treasuryAccount,
                    treasuryTokenAccount,
                    userTokenAccount,
                    setUserTokenAccount,
                    setRefresh
                );
            })();
        }
    }, [connected, wallet, withdrawAmount, payerAccount, userTokenAccount, userTokenBalance]);

    const getOption = React.useCallback(() => {
        return {
            tooltip: {
                trigger: 'item',
                formatter: '{a} <br/>{b} : {c} ({d}%)'
            },
            legend: {
                type: 'scroll',
                orient: 'vertical',
                right: 10,
                top: 20,
                bottom: 20,
                data: chartData.legendData ?? [],
                textStyle: {
                    color: "#fff",
                },
            },
            series: [
                {
                    name: 'SLR Token Holders',
                    type: 'pie',
                    radius: '55%',
                    center: ['40%', '50%'],
                    data: chartData.seriesData ?? [],
                    color: chartData.colors ?? [],
                    emphasis: {
                        itemStyle: {
                            shadowBlur: 10,
                            shadowOffsetX: 0,
                            shadowColor: 'rgba(0, 0, 0, 0.5)'
                        }
                    }
                }
            ]
        };
    }, [chartData]);

    const refreshChartData = React.useCallback(() => {
        (async () => {
          try {
              var d = await getTokenLargestAccounts(connection, wallet.publicKey, treasuryTokenAccount);
              var data = getChartData(d);
              setChartData(data);
          } catch (err) {
              console.log(err);
          }
        })();
    }, [connection, wallet.publicKey, treasuryTokenAccount]);
    const refreshBalance = React.useCallback(() => {
        (async () => {
          try {
            const balance = await connection.getBalance(
              wallet.publicKey,
              "singleGossip"
            );
            setBalance(balance / LAMPORTS_PER_SOL);
          } catch (err) {
              console.log(err);
          }
        })();
    }, [publicKey]);
    const refreshTreasuryTokenSupply = React.useCallback(() => {
        (async () => {
          try {
            const mint_info = await getMintInfo(connection, treasuryTokenAccount);
            console.log('got supply of ' + mint_info.supply);
            setTokenSupply("" + mint_info.supply);
          } catch (err) {
              console.log(err);
          }
        })();
    }, [treasuryTokenAccount]);
    const refreshDepositAmount = React.useCallback((event) => {
        (async () => {
            setDepositAmount(event.target.value);
        })();
    }, []);
    const refreshWithdrawAmount = React.useCallback((event) => {
        (async () => {
            setWithdrawAmount(event.target.value);
        })();
    }, []);
    if (refresh == 0) {
        setRefresh(1);
        (async () => {
            refreshChartData();
            refreshTreasuryBalance();
            refreshTreasuryTokenSupply();

            if (connected && userTokenAccount == 0) {
                refreshBalance();
                await getTokenAccounts(connection, wallet.publicKey, treasuryTokenAccount, setUserTokenAccount, setUserTokenBalance);
            }
        })();
    }
    if (connected && userTokenAccount == 0) {
        (async () => {
            refreshBalance();
            await getTokenAccounts(connection, wallet.publicKey, treasuryTokenAccount, setUserTokenAccount, setUserTokenBalance);
        })();
    }
    return (
        <div className="container">
            <div className="row justify-content-center mt-5">
                <div className="col-md-6">
                  <div className="bg-half-transparent sr-border text-white">
                      <div className="card-header text-center">
                        TREASURY TOKEN HOLDERS
                      </div>
                      <div className="card-body" id="chartOuterDiv">
                          <ReactEcharts id="token-holdings-chart" option={getOption()} />
                      </div>
                  </div>
                </div>
                <div className="col-md-6 text-center">
                  <div className="bg-half-transparent sr-border text-white h-100">
                      <div className="card-header text-center">
                        TREASURY FUND
                      </div>
                      <div className="card-body">
                        <Typography id="user-account-text">
                          Treasury Account:
                        </Typography>
                        <a target="_blank" href={treasuryAccountLink} id="user-account-text" >
                          {treasuryAccount.publicKey.toString()}
                        </a>
                        <Typography id="user-account-text">
                          Balance:
                        </Typography>
                        <Typography id="user-account-text">
                          {fundBalance} SOL
                        </Typography>
                        <Typography id="user-account-text">
                          (${fundBalanceDollar})
                        </Typography>
                        <br></br>
                        <Typography id="user-account-text">
                          Treasury Token Mint:
                        </Typography>
                        <a target="_blank" href={treasuryTokenAccountLink} id="user-account-text">
                          {treasuryTokenAccount ? treasuryTokenAccount.toString() : ''}
                        </a>
                        <Typography id="user-account-text">
                          Token Supply:
                        </Typography>
                        <Typography id="user-account-text">
                          {tokenSupply / LAMPORTS_PER_SOL}
                        </Typography>
                      </div>
                  </div>
                </div>
            </div>
            <div className="row justify-content-center mt-5">
                <div className="col-md-12">
                    <div className="bg-half-transparent sr-border text-white">
                        <div className="card-header text-center">
                            MY ACCOUNT - {connected ? 'Connected' : 'Disconnected'} - DEVNET
                        </div>
                        <div className="card-body">
                            {connected ?
                                <Typography>
                                    SOL Account: {publicKey} <br></br>
                                    Balance: {balance} SOL
                                </Typography>
                                :
                                <Typography>
                                    <button
                                        className="btn btn-secondary w-100"
                                        onClick={wallet.connect}
                                    >
                                      Connect
                                    </button>
                                </Typography>
                            }
                        {connected ?
                            <Typography id="user-account-text">
                              <br></br>
                                Token Account: { userTokenAccount ? userTokenAccount.toString() : '' }
                            </Typography>
                            :
                            <p></p>
                        }
                        {connected ?
                            <Typography id="user-account-text">
                              Token Balance: { userTokenBalance / LAMPORTS_PER_SOL} ({ (userTokenBalance / tokenSupply * 100).toFixed(2) } %)
                            </Typography>
                            :
                            <p></p>
                        }
                        {connected ?
                            <Typography id="user-account-text">
                              SOL Equivalent: ~{ (userTokenBalance / tokenSupply * fundBalance).toFixed(9) } SOL
                            </Typography>
                            :
                            <p></p>
                        }
                        <div className="row">
                          <div className="col-md-6">
                          {connected ?
                                <Typography className="mt-3">
                                    Deposit Amount (SOL): <input className="form-control bg-dark text-white " type="number" value={depositAmount} onChange={refreshDepositAmount}/> <br></br>
                                    <button
                                        className="btn btn-dark-custom w-100 mt-1"
                                        onClick={depositToTreasury}
                                    >
                                      Deposit
                                    </button>
                                </Typography>
                                :
                                <p></p>
                            }
                          </div>
                          <div className="col-md-6">
                          {connected && userTokenBalance > 0 ?
                                <Typography className="mt-3">
                                    Withdraw Amount (TOKEN): <input className="form-control bg-dark text-white " type="number" value={withdrawAmount} onChange={refreshWithdrawAmount}/> <br></br>
                                    <button
                                        className="btn btn-dark-custom w-100 mt-1"
                                        onClick={withdrawFromTreasury}
                                    >
                                      Withdraw
                                    </button>
                                </Typography>
                                :
                                <p></p>
                            }
                          </div>
                      </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}