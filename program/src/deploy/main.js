/**
 * Solanaroll Deploy
 *
 * @flow
 */

import {
  establishConnection,
  establishOwner,
  loadProgram,
  sendDeposit,
  createGameAccount,
  reportHellos,
} from './deploy';

async function main() {

  // Establish connection to the cluster
  await establishConnection();

  // Obtain owner for all accounts
  await establishOwner();

  // TEST PROGRAM
  // await loadTestProgram();


  // Load the program if not already loaded
  await loadProgram();

  await createGameAccount();

  // Say hello to an account
  await sendDeposit();
  //
  // // await sendCommit(0);
  // // await sendCommit(1);

  // Find out how many times that account has been greeted
  await reportHellos();

  console.log('Success');
}

main()
  .catch(err => {
    console.error(err);
    process.exit(1);
  })
  .then(() => process.exit());
