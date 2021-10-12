const userData = require('../../common/user-data');
const helper = require('../../common/helpers/helper');
const walletLogin = require('../pages/wallet.login');
const textConstants = require('../../common/constants/text-constants');
const homePage = require('../pages/wallet.homepage');

describe("Simple navigational and input tests on the wallet home page", () => {
  it("should have the sign in header", async () => {

    const signInText = await walletLogin.signInLabel.getText();

    expect(signInText).toEqual(textConstants.homePageSignIn);
  });

  it("submitting the sign in button with no input throws a validation error", async () => {

    await walletLogin.signInButton.click();

    const errorResponseText = await walletLogin.errorValidation.getText();

    expect(errorResponseText).toEqual(textConstants.homePageErrorMnemonic);
  });

  it("successfully input a bip39 mnemonic and log in", async () => {

    const mnemonic = await helper.decodeBase(userData.mnemonic);

    await walletLogin.enterMnemonic(mnemonic);

    const getWalletAddress = await walletLogin.walletAddress.getText();
    expect(userData.punk_address).toContain(getWalletAddress);
  
  });
});