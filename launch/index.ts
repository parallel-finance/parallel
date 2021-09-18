import { options } from "@parallel-finance/api";
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { exit } from "process";

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  const api = await ApiPromise.create(
    options({
      provider: new WsProvider("ws://localhost:9947"),
    })
  );

  const chainHeight = async () => {
    const {
      block: {
        header: { number: height },
      },
    } = await api.rpc.chain.getBlock();
    return height.toNumber();
  };

  do await sleep(1000);
  while ((await chainHeight()) === 0);

  const keyring = new Keyring({ type: "sr25519", ss58Format: 110 });
  const signer = keyring.addFromUri("//Dave");

  let call: any[] = [];
  const assets: [string, string, number, number][] = [
    ["Kusama", "KSM", 100, 12],
    ["Parallel Kusama", "xKSM", 101, 12],
    ["Tether Dollar", "USDT", 102, 12],
  ];

  for (const [name, symbol, id, decimal] of assets) {
    console.log(`Create ${name}(${symbol}) asset.`);
    call.push(
      api.tx.sudo.sudo(api.tx.assets.forceCreate(id, signer.address, true, 1)),
      api.tx.sudo.sudo(
        api.tx.assets.forceSetMetadata(id, name, symbol, decimal, false)
      ),
      api.tx.sudo.sudo(
        api.tx.loans.addMarket(
          id,
          api.createType("Market", {
            collateralFactor: 20,
            reserveFactor: 10,
            closeFactor: 0,
            liquidateIncentive: 110,
            rateModel: api.createType("InterestRateModel", {
              jumpModel: api.createType("JumpModel", {
                baseRate: 2,
                jumpRate: 10,
                fullRate: 32,
                jumpUtilization: 80,
              }),
            }),
            state: "Pending",
          })
        )
      )
    );
  }

  call.push(
    api.tx.sudo.sudo(api.tx.liquidStaking.setLiquidCurrency(101)),
    api.tx.sudo.sudo(api.tx.liquidStaking.setStakingCurrency(100))
  );

  console.log("Submit batches.");
  await api.tx.utility.batchAll(call).signAndSend(signer);
  process.exit(0);
}

main();
