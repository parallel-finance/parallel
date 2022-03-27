import { Command, CreateCommandParameters } from '@caporal/core'
import { getRelayApi } from '../../utils'
import { BN } from '@polkadot/util'

export default function ({ createCommand }: CreateCommandParameters): Command {
  return createCommand('Get best validators')
    .option('-r, --relay-ws [url]', 'the relaychain API endpoint', {
      default: 'wss://kusama-rpc.polkadot.io'
    })
    .action(async actionParameters => {
      const {
        logger,
        options: { relayWs }
      } = actionParameters
      const api = await getRelayApi(relayWs.toString())
      const stashes = await api.derive.staking.stashes()
      const identities = await api.derive.accounts.hasIdentityMulti(stashes)
      const accounts = await api.derive.staking.accounts(stashes)
      const eras = await api.derive.staking.erasHistoric(false)
      const eraPoints = await api.derive.staking._erasPoints(eras, false)
      const minCommission = await api.query.staking.minCommission()
      const properties = await api.rpc.system.properties()

      const validators = (
        await Promise.all(
          stashes.map(async (s, i) => ({
            stashId: s,
            controllerId: accounts[i].controllerId,
            prefs: accounts[i].validatorPrefs,
            exposure: (await api.derive.staking.query(s, { withExposure: true })).exposure,
            display: identities[i].display,
            hasIdentity: identities[i].hasIdentity,
            eraPoints: eraPoints
              .reduce((ite, cur) => {
                if (cur.validators[s.toString()]) {
                  ite = ite.add(cur.validators[s.toString()].toBn())
                }
                return ite
              }, new BN(0))
              .toNumber()
          }))
        )
      )
        .filter(
          v =>
            v.hasIdentity &&
            !!v.display &&
            !v.prefs.blocked.toJSON() &&
            v.prefs.commission.toBn().eq(minCommission.toBn()) &&
            v.exposure.total.toBn().gt(new BN(0))
        )
        .sort((a, b) =>
          a.eraPoints > b.eraPoints || b.exposure.total.toBn().gt(a.exposure.total.toBn()) ? 1 : -1
        )
        .slice(-24)
        .map(v => ({
          stashId: v.stashId,
          name: v.display,
          stakes: v.exposure.total
            .toBn()
            .div(new BN(Math.pow(10, properties.tokenDecimals.unwrap()[0].toNumber())))
            .toNumber()
        }))

      logger.info(JSON.stringify(validators, null, 4))
      process.exit(0)
    })
}
