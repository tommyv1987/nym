const userData = require('../../../common/data/user-data');
const helper = require('../../../common/helpers/helper');
const walletLogin = require('../../pages/wallet.login');
const textConstants = require('../../../common/constants/text-constants');
const walletHomepage = require('../../pages/wallet.homepage');
const bondPage = require('../../pages/wallet.bond');

describe("non existing wallet holder", () => {
    it("create new account", async () => {
        const signInText = await walletLogin.signInLabel.getText();
        expect(signInText).toEqual(textConstants.homePageSignIn);

        await walletLogin.createNewAccount.click();
        
    })
});