// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::store::OnMemoryChainStore;
use crate::chain::{BlockIndex, Chain, ChainStore};
use crate::network::Error;
use hex::decode as hex_decode;
use tapyrus::consensus::deserialize;
use tapyrus::{Block, BlockHash, BlockHeader};
use tokio::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// A hundred block headers hex string.
/// Network is regtest and start with genesis block.
///
/// The genesis block was created bellow command.
/// ```sh
///   $ ./build/src/tapyrus-genesis \
///     -dev
///     -signblockprivatekey=cSo6nWAdX4NhjUb6AXMNnNGedRfN2budY2UednwBs6UxVxmhEWui
///     -signblockpubkey=02260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a
///     -address=mkYqbqLTXNR4EhtxS7n7sNSW8VFTo1NU3n
///
///   010000000000000000000000000000000000000000000000000000000000000000000000623fd6e71aaec98e129d8b447ba7c6fe88cd27346cc556353d2d8232a2829f0a49b4a19f4dc3f0526dca905dcaff6a8e34537d04b450e0ac5568ce89a9373e301665c860012102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a40f457d5dd7caae6bf89a50efd13cf4a9e3857760f747e107d1184263e1212ef98cda52410bd822e92c44b4a22a2f6116a0df2b130afe315af6a7289567b32c1ca01010000000100000000000000000000000000000000000000000000000000000000000000000000000000ffffffff0100f2052a010000002776a9226d6b597162714c54584e52344568747853376e37734e5357385646546f314e55336e88ac00000000
/// ```
///
pub static HEADER_STRINGS: [&str; 100] = [
    "010000000000000000000000000000000000000000000000000000000000000000000000623fd6e71aaec98e129d8b447ba7c6fe88cd27346cc556353d2d8232a2829f0a49b4a19f4dc3f0526dca905dcaff6a8e34537d04b450e0ac5568ce89a9373e301665c860012102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a40f457d5dd7caae6bf89a50efd13cf4a9e3857760f747e107d1184263e1212ef98cda52410bd822e92c44b4a22a2f6116a0df2b130afe315af6a7289567b32c1ca",
    "010000003af2efeaaff9bea3add2827f3d9c91df4747cf430a9beb4d336991e434e0a1cdaa9503a1d7df3c5ef1725a437c3c2d7c2ae7fb03a9ea17e5aa40a20906bdb023cba4a7dbfa394a328bbdf6798d13cdf0147485344a9583506b4909c75ca40e6ce56ac8600040a8afe1891efcfad4108191b2b796bbe8ee2d4945e7c6366eccb20158e8792748a87dbaef8fae1cf70a8b9dfe529b1636f483355ef9f1f8453fd14e7db91c5ec9",
    "01000000356c119cdfeb761bb6dd99e2e327f453acd3a7a9dd69974604fd7ed66db0bee235179a3ccda9fb2e61e02ba915c91fd5955c0b150cbbabc958995ea6ae3802ab6c9e9c16f8b160b28e5634e5f0ca0871d2d831988116044c70424c42c2f18c0be66ac8600040ab19f6ba0b72e13cb180e7dc93ee8f296af662922382d4a9ad07389179733dcfb9b8caf3277fc04878685b4a7ce41117baa1c13322e0d3bc7e4945eb4dcb7bc8",
    "0100000007c8d67bcb6c11b6a3ed0bcd59aab3263932b9dc704e815b63153d44a752b548424f76dc5dc3bb9287e6d63762965a04da3b9d6dffd82ce2fa5f15ab58863f7c923b3ab4e5a189b36ab2c292f900820c4f648dee9ff2c1dd434693df2b20a524e66ac86000409cf4700f3ab221ceff51689b50c4b24ef4d1decd0c10dc01e62d74ccfcb71ba31bccab27fb8836ef6e61322ff9e8d9fc7852f18c8a7a2a7ba1f3b667b400ea9b",
    "0100000068a9554f5b6a1f0b98a567152e4b307b230562dc5983f38f7a4f16b75552e6bf20c93211b7931f162754d3b7eecf165e324f7fcc29083a08486902e3b8955a48d565a7378025143ef858f34d395686106cd84d7a27b48ec7abe0bf87949c3efce76ac8600040e45f88f770e4baec763cce132a953bd8cdd437949aed1ac68abd376157f03d0cc5881655ca1e16f7cdacefa875caaea78fe887e1aebd5c127adba9def89ee3a1",
    "0100000076612c30f58587300c2ef489a5d8c07f5052296886c28582d73244c858da55bbfafb3192717e93e86b930a8e32c4910e6740c62b555da84a03a3ebc179e96faf4e43cbc0876f0658456bf1ee5e8cbc45ed7eb92c96d12912f2211590e1daabace76ac8600040c7c3df3a16eab7fc12c263082815e75f3485bbac9f7bd98cd1cbb0d96a3867e90465898b235e123bba1b7571eca103db9166c3636c24e5a7774cfdcf5af0a76d",
    "01000000be9e13a2833e0de19999321e3e5ccf5ec93a3f777efcd112da22b00fe8fa82d3c164aec3351299fd67d79c4b339962c99132a995b487ff366352d691c2106dffe9ae5d984503a95e00088c47bbb8023eff9137aedd02ab3d65961aa666627d5ee76ac860004034e8a4b7c51db498e99ce93f12fa34c96d8065d76a35960221cdfe0d30467d3f87cc6fad1a319889ab8cb652608996aea013f70bf978594921e0fd1245d816c1",
    "0100000024d4751a7b24d506350f356ad48fddbc6c15248536084099c9242190fa46ee069c8d25ab2ede28db783e09d5e07d83090da96754fb8dfe1aabe4e821d738909a69cee1e07e13edb16f978310531a721a6a2ccb3eecd6ff68a343456dda22e637e76ac8600040ecb3fc56ca57fcb22b94624fcc464c6bc44efcb0b1f031627f5fe6ed92fa10cad9375fbfe3affd9314c01016ebd276712cf2e4b370ecd9fcb8585abf5d31a62a",
    "010000006f6972bd206d1e19857469383cb544f9f52ea70cac75119f0d3f01ad62fbdee23bdc6b836914e4d1008f5e7e71e447c9b13b1698d8223a27cdb15384d3c982daafc41c89108fe971dc9ff6659ceeb6204c0d79063f9c76a9131d3f5acb39505de86ac8600040e887c32ff385b62ba3c7c8629fc60c5a058a1bb8a719afce238dd89b6fb01702dd1befdfdcdcd23f48967c4fd32cdc9db8bd9ceb874815c9d3462f2c7e230c26",
    "01000000e952f452a4f0a971cf9103fda7460eb8a49e2916b746122649cda901a555a6f1d607f7f5081080090e11a1ad9fa3541215f7060c92d2da1bdf00fb0c4a779abf7e8b02b454fc63684c0716dd027efe6ffb4d02d1ace12c69472f071d72e9cac6e86ac860004043300d44986e33d045fa4615e299b1bde5236368fa416f22d45ba5936aeae87cd90cb8e02d6b0ad6f48613412785a25c6287ebb15a9bb684c6b7f86b03efa784",
    "010000003a3ea8d560b1f41fc85a0ce7b00076bf58afe34d6ee478b5ac3cde046562052312be4528545dbac2793ef54ae671b0ef26d352d2da358cdb609df4dbea814b7b81fdd202d4b0ccfb2ff2da35b64a94e215432dec65a352aff62cfc83763ec22ee86ac86000402cbf54e8b42c1bed88ce6cb250f779744ca15de99c0318da33fdbf812d2285c2370d8bcbd970d1de965c764b513ccdf1730b84b529536eab6ecdbbb104c3b9a9",
    "0100000025b1e422acb9f0a9350da2068eed0b4dcdb73d2f0ce3cca503d910d167bcc19bdf3a1dae0dd783c6c7958f2a07fb8195693fa207fd5c230e8f371e756238666e13f1d8c38521d38f60711c545fc97e9de6f36d86e97c5b1806d3a6844f04d7a4e86ac8600040512e2bb8aaa682d52046d148a41f5a55993c6dfe6c0bbbe977e7a18ee22ab523ed3cbf57d853b37f1a486133c734608697763579df180e29dadb2cb67a8df52a",
    "010000006a631ae8de3be4ac1aa23bd5eaa94102c277649ff6e2f537cd4135ed0a60ce6b26d30e718e070bbd9d7f847ec216a0a2815726d87d1ae2ae6daed9e6364af86597f14ee970a0f788ec105204d4263ca5827b97cb0cd50a69ac95526de1afe5bde86ac8600040a141bd0813146688bcd98c287c003a0e7861552fa3ee02c4e2793855ca9461c165142b9befa87961b93698dc51792202352d852a70a699bc888248dd809420d1",
    "010000005e7bbd44c1268733e994bb25f41c501f207f6c04f4976ee7ad74c949deba7a9b95a6b6f245ce9ecf3e549259ce79d39965ee8d798cc6913890c9a1c54578a158c6ccca197e12e064f6f0e8cb99d945d4c4756252629baaafd89cdfac05d687bde86ac86000408700069afc681a6c271a9b3a225ec717078ae6169d5db8c7a1b24aa138dc5fad95a846692607f6077771c5913d6d48edeb80cbbae9d2a59c235be423d96d81d5",
    "01000000776e9c77be912025b4eda197a11a8716c5b0a7e9176e0a77e54abf7043ab42b466b91f65368b2752ac6611e181d7fd6c2d4b8febf920137237e12b5e2fb8de36514c380ab675113a6113fdb8db1135664f7a63c64062db11343d6db8e043ea18e96ac8600040afae8151f18c71880988292af30acd1bcc42601c7299e894e42c03e8fd1281d612b3cdd07bca9d418da8d62e69f966623eb97c44244fce3a0ad96d81bab355de",
    "01000000ac7396e40d454968795937f1f7942be732d0e5a5ae42ff2989db24a446a32a78b7f8f07cb58377a011c7c3c760a568749f3f909e71615cdd834c774e0a7a4af2dbd151b77c58c2fcebe5a90d4da421ad29b00f2a559f1c331e5326bba77b5fa1e96ac860004088fd8c5be15e8cfba060b2577566a83a09d77e21491e334dfad423e31933ccdef2f868194ab56ce04eeb703ab9299f6f4759de5f96d83d9ab5b9af16154dedd4",
    "01000000d065d7cebe8938480324b73942687027329f5b0f13349bbeeccf5890149e846b8f51a5aa8ea24e3b03943c6b15b4ff8e41d73179418f9b6d44a9cd09ca01709638de3b4ce7c5fc3d40a8c8b0a7b543c812f217a5a99e72e0b568518bd6fa58e1e96ac860004004760e2432f2865b69d2f1ab96b97573922e2b7d92c705e31267468e8f372031b6401cacd2ec64030163ac28210c06835d67c6aba7305bef8023a4213f7f9c4e",
    "01000000eeb6d324d0ea7464e41eeec94be26481d2b64391ec683fa465fedda693af2eef901042f9163d6d9f1e01dca12f6b4e5686361bebc6c76428db694239210aea9b8640c7a076952111e1062aedd1113b13c00b04a92835b7f8150413e3a2e60299e96ac8600040f6d39b288e0afa0ebfc7c6ee6aae8c47c2754dee146719ca1302bb4a1885b5a0ba862787e83ed131f6fae07d561ed0f4b68b0cc2dce8793acd6a4218bf02ae4a",
    "01000000e25d60a9c7ee2728517dabe6690213d740633fe62d0865f00b4d251f3e90e8bb52d001f64da884dffa2d5967988986871be2df2830e9ed5753826ee3cc0934cec4051ec61b28714a3e15a79351cce3a376525199e289189bef4550b7f524b21ce96ac8600040e115382127e484b799e3fadc623449463a73047f6976ef88be36fa60d496e7d8994628000c1be8828b34d72c3c985792e68f62229413e368b85533202a48879a",
    "010000001325b86bf2e202e30e082d12b28b6cdaace24d4fdf98326481a7d6ae1dcbce60a5bb724db02b068c95aaef23fae8ba857ff14dd86ded247afc6ce0f2307b9200e8f1b3ba5bc9d4829c05b3f3439fadea696729df5498ae826e38a4c630adce3de96ac86000400bd7c349035413a5a89536d1c314f18b7b0b6c98ed4c84aa134f4cf0e3c9c81cd84ea70986a6288631ec055a7da54ef38ce470b6d547f3360ab9f1ddf1cea5df",
    "01000000e7e50a23a955fcc9fe1c1135faa45cd2fa2f7137d41cd5dec021fce86ebde63e5eb1c82c015d5db6d89f8304e29d86e0f2cc1b0e11c560e5b9bf2a831a454743dcaa17dab78b6d75d943b8552007cc5388df4aa0ba78de6855e91f024b8c322eea6ac86000409d276c88c4fbdb86b9f93f893f16b653b46cc53b42bafac0bf2e6a8ff83574a5eb4b00af16e5ce67fd0759d94ed3e434eb8ac9e4c41125164c0f6cc6f55a3a82",
    "0100000005b7681b61ed7e043181725da09d132a7135e6addc133962a365e12b8d6cdcca527d62e0254eda7142de2eb0cde09b55a80b9e8c5cc900a93c63fea192ab3a2df85071ef7caa7b2bcfbab177d470b41ec7a509529ae8b35e5eed4b2a06e6b584ea6ac86000408ce702f527c096bfef172f02e9cdad7f3d4b341c44e830fd018885bef814cb9600b73b716c16791406476c185273561fa7f597e8cf0feea20ed47de0c5722f76",
    "01000000cdde1268b9ecd74d1315f46680b7c92e04bb21131fdc63181253e62615508b71088e604c5bd9cf7c7587c573ed382d9c3595af4467f334de6e40d6dd3631a7e375d5be5ecce6af577603f5f581270bcb720f64054744d8cb26cf11b26661f032ea6ac860004087acf7ad825f13296dddc58d3c526e1249b8c61423f559abdb279c95c47b9f33df7198e67327fefd2b7693f6f19763b852a895f55231d50e243d2e794a6ace0b",
    "010000002274abebd8d1b602570d70959d89d8a6a58be6058e9706d5f3473933fb85e16773048bfa7055a22ce90bd120ba141f99eb3e3feca3273b3abc0b453c5950bf0ee5d09f8fd8745b8f9e6bd71fa10afac6f3d0d6f99900d1c38f6ff6be58269525ea6ac86000403284f1bab15d79e949a283e1d21bc129b8059fe8ec6db46c657c0cfcedbf5524b617d2c91122caafe7bd4086fd60ce53727ab1b50a641786dbedac4b20af6ed8",
    "0100000085a3a05eaa74a6b20f3839a16a3369591da3f0905ceee1395c3aa44682b5a3b6c4a1952bb2f97357ff0b10dd3a48b7a1fda29aff7f925be4ac2efa9f9d877ae25934524915d45ed8f2ec37da17cfca74d09a4448a7de87842453c8c5b9d95d1cea6ac8600040fee04e4dd3c002212785f7663c92f99fe7e65e5c94b05060d1381ab69560746ec6d73ee8ef1a6ef81db5763b19d64a835b7eb18f0ab076f2b0f1109c3f83aa0c",
    "0100000062c32f80bddab2b2afc26d905e81fd54321dc39d95b4b622b5d483ed3193d02db87723a9323b2e8d333fad345d88ff77234c3f9888390cbedb320a8867df2848c85bc4d5f9c75cec5d29d9f5b3b95bb4479bb92445e7142428e81e5ae4f9cb50ea6ac8600040aedafe98239fd79ff39e46426e26d6e319164be03fae9d4def86426f7194ec22ec3bd555ca98b21693be66f42a2fb74fdd98ea9568408c6786ddc69f3cf6e257",
    "010000009567777ca9b831ec35980c55c8044f5063d9c2fd079b16ec900ee521b495a36e3efc39756a583ff591f0a1a134e5b4b817ccca5764c1435aa6dd956e1ca5767b4ab60f8ad4ad06e38f4bdee3c3d0ad9958c323d6825c5826c5cb2e2b42eada65eb6ac8600040d9149ce2661f0bbd5923aeaf9795530e3fb15b71404dd28ff17ba1653610467ea866d98868cb2dc97f6acd48650ecd0390446e8346f29a22d46f21fbfc689d78",
    "01000000c5702e6d27e0075325804581412478d26a3a8916ea522dcb0dc774eee88dc2f738c273bb45588ecf36a01fbea583d072598115131239ec2328ea9a858fa7e2dab61ad76b7353ec76a652581feb886ff8a817fab5bf9741c2acfe7c93cb3f45c3eb6ac8600040d14d1e11c402121248c54d1823b8f24c1361c2f8919abd6e8ada887993fa630f677bb82e1516a86ba42ac4728ab620c2341eb4a3cac8ff286935c584a1970af6",
    "010000006788cdbffd83d11f1c145894bba68b3ce5714e435966377c39b43c4fcba4203347bfffe7463feb183555b63125210fdab3679bcd4454b58bbf3e35d5c6105858df49a88af9c22be069889d9e121ceb9f97c2abc4fdac7d9c68c3715558521f8aeb6ac86000408b1d7121a67fb2bb12518eb6aa5bd05c35871cd553c42caf7c5dae962404f53d393ba7ecd9b23b1045b5acc95970cb130fbcd980d0192ffb3a1cfadc3d5651c4",
    "0100000054bc398e26236fae685d1d67780c5bf87490386e2651a6da0cd4d42ca9c17adaeeb9d005c63f62a68b6d22b8bfba87f5c0711c7cca654362e6b967e614a3299e28a118de10980baadbdba88330d99284ced7940147d8970f92ffe17fe931f183eb6ac860004030b06f3a698ea14b6d3048978c2b67eb183cba008e5e54f051fd5ffc09a46bee9ff285dbb1d402461e57971e8bc604c44bb1d6c781a5c2b6e19650ffa8eadf1e",
    "01000000744d3c21a4b3fe2b6ddf4cbfddd37ba824cc3ae4e2b66d49f7d68d6f112727de3d211a2c2911a181737dd73b1a0a9f87b925ffad3dc6e35a0f9871d36cf1d950e437859abcfea34bf9c4ce6965e2be16d6f3a467ce0d8e7b4ef8c1569984cc1beb6ac8600040f6ceced2a3a696f3b4e6bfb58a173acf9f46ab6972ab2878477bea8285c20628f34970ac6e34bd7aa030ca76fc29c13b64239ef1f8e7a13c1877c4cea6476aab",
    "010000003b7feaf70e2ca73ed57b0fd05aca63b6e1ef366c8066d6f227dff3f45680d44b17086c9640db50bff898bf5d4af6128a01e0864db13e29c9010df22f79846c591bdf93cb428d0dd3a8bbc3f4e052a3850911582f61ae76579393d076a5d04d4aeb6ac8600040ab7f1d258234e5b57a2d30c2b531120003df18eac7083dd3af51d6b3d1215c3bf2c08c60308fff2fd6e97377363bb1ca9ec55f452d52db4d1e13d536ee8e0e5b",
    "01000000154af4e6bc39ff56fac0f8b8da36fbcb63b3ed9f01608855531c23ef0a3fe46548b97beeb0a805aa0e730d43ea756c0f7f68e9946de939077d7747a3b24521cf0a348cfe07e99a473bd114e8abdc27b3033e7dbf01cd1c06a62d2105a1ffa5c5ec6ac86000402f9664d5bb8f32c4c83d78c212e71115b44249557a2f0fe2b804b6703fadbb125bdda294a92d8904c97b92a33843e3df7c6719f1e61c642931a9251c5c31f525",
    "01000000cf673e9d07023db65c8dffe9c8267d08675c479c1788f50f62808bda40c8aaee119b7d2965018f1b0f1a4411a406d74f317a3916c260317a33eefab580ea8a8dfdf28e95f9a2193b67d8abd4585ce149083b0c8f00e36c5ed361fe3015c73c01ec6ac8600040684090563b6a806797e3e35d1b09fef54b1e1b017cd04706274299d2a5a7481a41d486a7854e300135381f82ab2166125c3f1a7e88a72e36b0843b877ea128e3",
    "010000001c98fa2397dc3bb57735ccba32b6a7cecefc2309d7afebaeb50b25802c476e10ae995ddb03f0be688e65f1f0e9ea01b5ae06bc0871654023c58b8256c12f6bd6640f373d386654be4fa282894334d9a1ad721a602d54a7cd342503100c20ba9aec6ac8600040fbe44007a94eb03ab6598496c29a748393618a83386d5684ca1e188dbe7acc93b17bedde72f8b4dd6ee8f3bbc2127f02af12918b5e6232483025d6aadd5f9888",
    "01000000bc7eadf70677c4f869e26fa27ca02e912dee4c0dd1e3aeaa6d7f08a327a27e757a1352e8c52de7e32bd71a6242e6563d9985b40142c74e15bc6e64e130bc57b1a416d34ddf74c709fa34fe9234ef71f8fa2c66e59f1fb496ff534cc5b97a0a33ec6ac86000402681d89aa4fea6f8eb52b396547a73d71d6053c352517a00ae27a5843e7f9ef87301c23568763298c98cd1485942aa330f5d0458268df799edee2eb70f082328",
    "01000000e4ac06ee63abc58af222f02ade7b9c0a59d1cd263f05d15b140f763bc17abda8f71951751b2cdb0a8684c751359eb7c77b14b87198f4eb8f165bb35ea901502d070f380f5e9e59d1fb8a4c7841c3043a2d20389a254c7b616e787848034a95e6ec6ac86000404d9efa13e76433ea9ba778c68f76c7493d6ce3c497aad35709f85e4b88db7dc0b11ec9ac3552cec7f0f06328c71407b08fc668663585790430870f04cde4390b",
    "01000000edca3be1c608c12748b29164871a8aca70734e5ed153681946d7d9f23bc1539a5ceb9d7886c7cc1d84a1223339ad707656a516009ce9169a1257ce37c65bf9128ab47116e0569c8c38e13e785215e349703967c2d24ce250912a10631717bb8aec6ac860004095dc362f8a6e88b5ba9a728e6871b6598feddcaa612aded915e5919e9410c244d0dd516372cf011f5ce34c9ed9eb3d8d6d7a6f271f6cf7978003ec811ee8e968",
    "010000001ce088ea523743290ea7bab6d0588ff948f98302dad3481882d7a5588f16d8e69a198a753a2b5dfb66cf67b0f65c3efc5d8faa7125ad49af7d48dc71db18c2bd2d404bb5a766c02864cb87564e1d51a551840071189d0a781fb1166023e7c810ed6ac860004058e551f5adcc2657dc735ced3e15854a2444ce6a8fe337d3dd48f9392322c99e27f3405780570ef782d22d9840c1bf55c2c597e22d369902ead4d3f29683dcdb",
    "01000000ffbc49c77a8064d3d9e68ad2a6bd5a1448f5c5472b2eccfae49026b73087bd197c1152b28ef80656b08611e18409c0c40886a089abaa606bd5281b274ffd0039bd7934e014d4d001dd59e01c21d553f22027ada8df57b6bce3e3bff55fcd5ed2ed6ac8600040cc5091d115891b353899c760df0bd9e3460dc9ff96a3e7a1cd7eeb6469c6470d218c8920b5a21eb6332a426c36cbb445499f3821f947676011e1e5ebadd89ff5",
    "01000000fd85c213ccdf50310a7b1633a8620ea384f8f1f74c807d60c0036783869bfd3eef9b1a08ede33c4919fff32efc2fa42576a2b17c71727999cc51603f2383e1a27bd7dee6db1c0a3c540561337fab570a8031899d009083108a85efa9ed6cb507ed6ac8600040e211950ad181d9aef5f27eef439b2336d134b173caf539a0bd027189a23f5aea7171a062e7fae53d79b05c8c4497d7f4b1ed650d8aadd7ebf0ac36f2a9267e72",
    "010000003e6b9f298d42158017331852098ade4afd5f23803357706ec3897f5cfe89bcab0638e040d88d547796e6d6ef1d8a4ec0b3411a6a78f885623955dd58375860d1f5c535e563faee2dad477b4038384fb65b923b772e5793e71376f33c536781c8ed6ac8600040ed98fa9df98c3e1df6fec2ad6bac37ec95ad23f572bd69933ba887b4ec078bb3daad383e5697b82c6f55388a1882a805d068d8792b8970b6c89da30365dcdf59",
    "010000002c886d270b8bcec037613d6a7ebef4b26aad977a2d1b7c8129c7a1a205d7f1946564ac5fca71bfe8cada9f940186a454496c3e75dc431fab6d52cace9e53b17b9d5eb2ed51eb61c4fb4477890ec09cdb3db105f4f05c6a4026d26a2c8c730d15ed6ac8600040338cc4df336afd7b2382a6e3dc2fcb07b318a42628d3f9bccb9949f5a3efb68df5dffe19b7c6db59551bec7f18e7be7bfd3685aa94f61681982d8ded6cd5596c",
    "0100000040205bf0176f74db38a278d9d99406ee9b7a6883f271fc0d3739a520bfa9c22eb6db3aa6793cf4d746d3984139b6156dc75a0e1f7bad93173b6cf0a300dc40fa5a54c4816fff9f4bdb56322ad476cf8358c760c9573cbf8705d2dd1ccb4c8a76ed6ac860004050ca27e1bcd9895e32d355a6d6630ba855d3da628603a6f86a0a63a75cb4c16f29c34456e503842cfea678a5c757609f9f02c1201ae37c1c0feba4339e79cd2b",
    "0100000069f7090723d1ad8d04cf23823b23198e06a350958fb48c8be93207c98309dc978c9cd03aec5271dfe049e0dad2beccee869baff60cea752d64c3d73d88896945320259665520fd56e6f2c4e054ac677027ea45346f78289a07266af46c7f88e8ee6ac86000401dcf075d5f8887065f7f42916aae619dd866e57d645152c24f5128d5d67f5e62e6611c1c24d3616a13241910bcb2f266059ff7104cb183381871602c87ab6e77",
    "01000000ef8dd4b216f653b77eb476430f439e0315593ea50b890d6ae18758aacfb1dba753e4cc17c37d820f6a61c000bac642309cbbf56b19d9ce4c66e48a223cb925d5d854677614c559e0acd734c2e90337b7e16e8f54b23da1c95c4e27351e433590ee6ac86000407d6370b3ca3966aaa4001f054ee53214fb308d6da6489e0a9ac6ae12e72805b1d16a27d5c31eb1aa3ca9eb6820c88634d2c0e7bc6959ef9924dc1716d1e8a0e0",
    "010000003a1005c2a4979cbc7ea6d65a4de0ee1f879ae6ba0c8c90f9192d06ed0f55ea820e4cd5073aebe630c57f8c2897bdca1969e5a4aa93094b2b621e3077a6a9fb306cd47ac8f4357edfeaad675f2a5be9a74b818bc230ad7c4c6f41313a2cdd486eee6ac86000406403b5303ab87957c192af61172620f8b45ebc7d5412cd8e3fa6c01e0613d85440fefe765f3fe85128313c3e648f88ad909eeff958d6b248b663d5adc4fb2913",
    "01000000a3aba99ff2eaa414782782590dbf14244e0b33a38c62dd602b8fd124b273664ab18eb38d9fcd9d198e8473464b8b78cfffba174b5d5188f7cf64e7cc599f78c270546ae485da918a5c1e87791c839061dac331297b314ebb2ff2449a93dcb34fee6ac8600040023281dc1ae9b37fe7480f279dff54ee45aed61404cac3b7fe95fc9bf774b3123bc0fa46bd25c4443273af0fec18e3918dc0e31a70115127797283e8132ee54c",
    "010000001ce0ae745629cc8bd9a6ef8533abfa4ee2bdb466635834775e85417b3467107873cf45e744487842354b56c4e61ace64bc8ca68bec4b8578d3590374f3f27f5123af7b62bbebfea82d246c3cf8f05867fb172e02d59810dcfcb25d2899473858ee6ac8600040524231fbdf7a8517d58e02804865f3cfa1619ca68201bc537b78674a2c1b7751950dd1e532f3e9fe6430e9090160e449a0dd77f9dd01d6c3e07ad016bef62472",
    "0100000046eda348e675186c1173a7cf0f903a45a36bb40dd5f2820fd182ce2000491c51bffbe876dfee9a375104e2549da0222eb4e6662150f36af1e52d378df107ffccc470fb39156b1d300c781fed708b8b7d3d27f69d460026897433c29cd75451dfee6ac8600040ddb6f3d8725b251797e5a9f4248adad9fcffb4b5958b6726bbd2656e3637e9aee589a55f3418020f89e54e9f980cc3ac9bed1d979bbcdd23026cc5a09f85ad89",
    "01000000a22b08183191e9e6b014daf7810830267bb91d898b7b0463ed2dac9c1b8f96c223059cfaab463613eb7bf7dbaa375a09da4d6cdea781ab658073e59e6c51f5b5c6ac1ad4b0e7152ba06b644ce8ee86cf0b55b8f7ef0ba7472821c53697ed2b36ef6ac8600040fcdad469afe956b0041d7ad640957432fbec75c3094995d12937ea519974913c4867d38448469f8d38d30c72e588c423803f9498e67ad6d74a54b41c41ba2863",
    "01000000b4125d4aa55b8fada188ac1d88947d546a77f96907a626a4b681ee7e8ec79af380bb55846e410d720c287091b52081f6bb4737d556ad9fd907874209a37cd23b2d3f2c66aaa8d35111460f29b77d2a4996eb43ac196996893e7dd1d04c66c444ef6ac86000405928600235684832f0e32edcc4a1959c750acfe8d5a4853110349e79c642f49b1dc10ad5b5228330a2fecb8a7d69e2def52441bedce9d579a1bf40d9a271b11c",
    "01000000c66415be80125dddee20cf7ce57eb95d10a6366d9d5c6db89a74d4fa1ccadaafc4b203dd531a62db5784f4a55d64aaa4fe4a383a4e9a2f1a70e7864a57d57dd12e32a955c15198ed263cf1a94270d1cc0c19c3c331d58df65aa922aa29db3713ef6ac86000408186037ae886c901ff338a860bc7725ba0756841c0dfd3db5b9a61a55e164ab3478cd5321c833ed963423ad02fa7f362ea5945cf8e0f8e076b8fafa022f72c6b",
    "0100000044f611d07b28e59f98846e17d4cdedcb6e427f0f081a43efae91fd65183ba8a0c60acdce8d605e4315bae846b4dd62217f122882dd645a1d8b3740a0e8e3f1461b2b467dd06c3e5663550a47b1c2af97f57af4807e96244c88e2fe7a71884a28ef6ac8600040be0e7975dbd8d43ab4e490e706be25de3939d3271675fe717de9796976b719ce2ec7ac737595734b79a552bd3d9fe3c78933d862e5fe005eeb4a8208dfcde8e7",
    "010000000f64374519c5e0e954c68c743ddbd7ffb6c965d5d6b97f3049c2be979f23f914a6163b66dd21edeb2853975b0c371ccda90295ce8160e1bce22f3fb54eebb6ec2fc181ca9220cecd2d6366496f2bfb85df11fd229e8836963d907bb32d5c4a4cef6ac8600040107be1844a512dceb47f9c346b334ecfebe4afa83006ed2a25dc027c31b284529e1c80e0ac34dcc71801830afae8c8f5c1cef941c7d1de815865844cf1b8cfdd",
    "01000000a1d33a1c1fb0df18370a647be2f9edc93a230efcaa89788265c45cf694a29c468aee61847507dd0593d9d37a67ef420b3a5f1b9edfbd6bab00c20b946817e9bc93d6eb82b7dc2cbbd7be512742576fd763bf5dce00b56b062181bc94d0ca8b19ef6ac8600040c730403d7b83ee98ed461a914fa088e8301c9a68c88cc94c3c291ccb6b09dc9e119350402190cfc0381bf5d338ca6f0c2ce0724538d062ac0a5a90576f0b280c",
    "01000000b591440a7c4ef0cd71457814a304419e87f54447bd76b475ed240ebc830b8d84d8bad2944f7bb57c74a63cb979459d1b846d64e21df7a1cb96f1ed3bd621b0c71096117c967c6b28f5981d719449c65490238c2850e841316ab87e1e41cbda24f06ac8600040707dc0b553c894cdbf5274ad0b481ca8a8a7377d8961870e74a8fa570e46d5bbae054ca590b78c6f369e4038424db61ea9ba2c374ce638ab7ebb2f29727ee653",
    "010000002501f35281fb1647da83df8e04962b7262db7656d5e14a7b29a3a7cbbefd0a68f926c93376647d42bfefe2a126afc9adee7d90a088ca050682be9c38c2cbb4002b2884cec6152897227454b6905928cca1bdf57a11fc3e54cd87b4f0cd0317ecf06ac86000403e037b02f50df250797a57901e44df861369a84b33fbc5966fa9e9d54add9afa419216491be7a479bb2acbef80b5537db242b7e06afb7796af0b6f1f4063f068",
    "010000009c6b2fd46184a1897a1adb217fd3f8c9976f3c76d4210d9eb010b956e7f33f1d92072d58e0ad64fc5354b0e36c315521b95d0b3d4f56f47f4a58dbeae78444c422dbd52e47be4a670872570b9a7db82f4bb8ad215e4bb15216ff3e4b2cf4aba0f06ac8600040d270c3ce2156689afd2de72fecf8866a4923f8c2170ba2373fa0e108f14559466ea9a38754dacbb84102773821b970316404e1270e254390ede2c5494ddfcc92",
    "01000000fe6bb68f3a9015400dfc0e0af1fba977ad5e7d02790405adbf978faa7fbabc5543db5fe5a585064de423a419335c84ae97217ba9c4d06d1837fdda89e674e3ef6ba4c05557c8880eb35ed79c82d14407a16ca49bf0bd3d693e59cff7291b5627f06ac8600040580b1a026a3667ccd42bf69be7ffbb81264163bc5122af9e16da91f4df4c582884ab7c69b253478c2142057212e6508714add1c109d4234867ee3e3367eaf2cb",
    "0100000012b43f665c7c2e061dc968f178eb43c3c1263049c86d996fd4572731e53c8b98135f594dcd2f0ce79f7a36292c4581227097bffaf274408b17e9462fab3b1dfc2c40e040716546cc2ff5e08dbc9807354869ac1036a6d8f6d8ef54e898ed2cc4f06ac86000404186e2ec74a55f68b92dbfd572389de0143b528a7d7f8c2cd054ea7a4a33dedc40d1bdcfda3c66511a6256b7b3d5f74f0b76dd3041f1966f41107032303ee748",
    "010000005afa9a475e304f3e670b7cff9e06a1ee1bca53eadcd85339d7647645bac66d15d4246d326dd2738c11f379d2da3e07658c9d55e0b46a36dc470b9867c3ae94fcb38024683f3a1c2dbe803db3c87428f660b98ac75102f2304610fa3dcfd2aa78f06ac8600040f77c7252cc5a9c3ed5e486f26f80438a6d89e0857d5cefcb1babbf8f31b7eecbc8e0abbcad83fc9a4e4373de39057446884dd148e527d3d08b7266de777a60fe",
    "010000002509ab7ebe288985699bcfe586d9d57e8ab434e1e03070efedeaca47bcf82654b34a2134be02d375a616c410f34f7cffaa74bc6df83578141dcdebd4d5be12363891f847ed8fa0ce5fd19245b247244583c2599b1d192695cbfa889e940a34d4f16ac8600040c9be8bb440964101138ca9cfc66d1448e0081055d1c217b3849011075b8695e3a1d34120af01eafa83ec2e66d90ac26d2ad14879d577c8bbd4e03c3848d47b24",
    "0100000057c3e81dc468e02078611b6a349978dd12dc5357f1809844a78916db48bc483f76d046dc44b628ad62968e62652c73f77c39a7dbee52c8bd9a4b750abe6fd6cd1f51313cd4f0bec3d71864d32903850d36d6681b99fee88f8d21c3c5339ad614f16ac86000405292c4033930080778a958848c0c3acc8218a1158e9068eadd6dd08cd2bbfcb25ee3e532c0e1721e566b84049e54a7f2950ccf13dd9ae8481c5f64fd0df54e45",
    "01000000b1793b25a33b4c8ab1d3042e4d3bde14b5806a0394129937eb7f354785d2fcef5cb7d43ce5760cac341ef85c38742bd103460d5852509b3d2b183a47b8641cb56f0f408c11adb17c6be14173b62da3114375e5d660b6259f5775b166995f6ee4f16ac860004011b76c8e6f7bdfc09695aa7586f1efe419b900cf5bf46f99f1d39071598ff79b1269d31263751fcb64c2514a6cb5971384fd2b026c65ce536b9ea78a8be1b34a",
    "010000005fd1bc38f490edee870cf81b33f094882b1e0fb2495a76f77eb21c07699a536529e006509347818556cc4305c3fc43230eeb85846c3bcac343d429dea1435383e544eac7f061c42e013dd1002b6233944ac806bafa97f1f2034cb19f7359cffef16ac86000402ce75690e247147c94c1fd1a8f2922c635c2cfc47ea6da088005f575b6c4e6ae29b49df233a64bb0f41401e4ecc0b1d3f3a390913d5901515c73f6b386bebc0e",
    "0100000029919d32151b9a8c838d666e3787cab739ac5595b1f6c444036546c2c5e8ba1cc6fce6285d251950bf1b297486c6f5b9ae9c5cf33ff11e95c3abace8187a73e279818b869b0f49c347e48cef23d108f4ef4098c247e2a6c58b4f8a89c5a8d869f16ac8600040433d2f5ff28d038e0bc0dae0baf888b107b1c621096b44a40c4ff6598aeb142a1e34c5b999b84ec73006f0fe325b7fce5cd45cce48d879276d813cca2329ef05",
    "01000000f6a1b22ee931dded92784b702123b404afb1df5676bc0175f218ca41a466a05db2ee53becafca677aa835a15386dfd70af96f2db252898e689f255779557d60dec9ea8c41ca24611a4ae56ccb9c70ec64f516b5a4b328efae8a33599e18eed81f16ac86000400cbb0ebfab4b130ee4081ca9488d9630e32a1827174a368c278f367780a7513842918f672c348abaf5777a3c14c7cae871bffe7de0cdb3eab63a009248d96963",
    "010000003838fbc1be5338092433e13980eac1368452cd184ffc97e211819cded2b479bc46998e954a57dbd8844222bf72897901633096edade71aed01a42a2fb1551449790331a209eba48372e63f725f4263a3790b0cf5eb012f938708566966763de0f26ac8600040da91256feb0c40f1e6c964922be25550a4d1d8e0eb581214ab1ec2933d964cec477b944d02698875f32c3995a877b6586aedf8deef3faa08943ba55ed1e1beb3",
    "010000009ef5ade70c8747f6438b1959ed7ae3d09e957cb738a69f5496c8bbbba54b04cd21769f6ad1d56c1cba2df67c1065b47f75ae846a7f2750ff14e9c0d54e6d09817031b44e142d42e4645634e7ff2c797ff9fd2c47c279e6cd0d8e22df38e00311f26ac8600040cb5e65ac00cc6e89c7528886e7a1763d076055373bc00a366c20f6b6d64fdcd75e468a8e654ae3f9cfe38ccb0dcbc7db6c4d77395e005866a599a5655ab40d67",
    "010000006d8559d0ba96315eaecd47cfcb9c775de05fef78668710e52f6a812574445f68eb24026989f60b84fee0e7365717773c281b023b0509e352587cb04556ae6fba865f46566e80def0567d5780e5dec94b89dc572571eb697ea987abcd231599a8f26ac8600040433dc322821d082ed2ebdc8429924c0525b0df5eab765852bf3d9e1993b9083df848da733caa825ea092b2236c8b40bb74089c684340abba9a532a37a25f9223",
    "01000000015df7af611415d15372f6685c18cd47bb33b3a011710c55a6e2cd6fc5ec5c7eab379056ab0fc75ba02129cb4fe5d493d0ef8fa7f8b7ca89ad7f09f89eb74803e1fdcfc9ddbe7e7974c479cc9673d732deeddf061b6ed83c106d52c0af542835f26ac8600040d8d9c817398909459b800eb1f50e6e8e9ecded672fbec79c58c22b4a01d1e1dae42e5fdb7d78066da2cb5e1c70b56dc9f70c5f363f352058955cfe4bcf34c0c7",
    "01000000d5b9636d939b4a388ee599b10f08e86d428ce6bd14ab7654a5f3aa0d63411a3ac29a58ecb5c44165d47698f799333c46e0ae312d85db835d81fe22cfbed0b5c5cdebff96072388d422550b40e9e043a038b13012769fc4e7f0b29bcc976393d2f26ac8600040b648f721b409c82a9f1fc375fdbf847bb0ee2ec95dedc20f6b1d29530dd338bc12af245536feca1b5610260efeaad9c2f5eac80724931b93724f79c52af16d97",
    "01000000caf450b4dc8102da5ae93f874a3afe36e44bec4dc56830df701f3b87a10960b63da079772ee9ad3626314bf0e55470c4e070abce9ea1be0b0e1fef377eef230c7188974bff5fa8f27a65e8185eac9215095ba69f52774843016f378d6c1d2227f26ac8600040fe58c7cd8a6bf2459727e98ca225da59389e582cbd4a2fd7cc5ff3d9ca424bc1d570dc31de49f0ed246fbfe6daa2165569b3844c320d32a5af11db02f5fbb2a3",
    "01000000b2812235fc3434aa85c1944314025ca4099b17acb4089b208a55b9b2472e82e0e24106fbda5c175e3dd7aa2469f53614277553d077a0037e67f4c6573c85afe66f697b2dc302ddd96a9b16bc6c67cc689b682b04b6fbf00daecd080a10fd1ef2f36ac860004046eb1acd1c38e15625b54f06b432522c5f3cec1a1f207d4dbf88b659ddc33b04b6ea031b6e2e2e73a181a8f28efd1505096cc5f974c82bd9ec0c8cfd449d6249",
    "010000002bf49a9358d3c2ba2037bab6db44f56547561ffaefcb4dd3a0b25cc370f234413b561abc2aa1b0d4ced22285995a5917b00af954b84ada852b3cd598ef0cde8afee7c1a1248bc481e523f06dff5c84a6ea9eb0b2b9a608409285b293f7793f3ff36ac86000407277b4498deb3c77486ea5ce8063ce8d6672d70e8706fe634909099a09a2e8345cb54f09e2def69c38fb81efdb957b1c38d1945fbbdb20ff51d9fe1100dfb021",
    "01000000c9589ece4342217cf233f28186b120eb290c379b05b398e3d3d6545111734dc7fdd4ce0f025de67e6c00cc28ad965bf21624c8386d7670c9eaf173b5c1b4c2efb2631e3c00d2b0ecd4e10c4ab1b6702d6a45a890601dc045ca8d428c1a3c4c5bf36ac86000408a515eb4126480af36140255dd20acaa578f9219e97f44bde8a4ef9343d1d488b95c4a905515974077b594a5247fa09ffd5db69fd3db8d4eabc6934c5c46bb36",
    "01000000ef0d89f496253938f035bdee0e1ffb1c8f0e31b1236815bb6eebf6736e45c83e5da4d1a5303cc2aa106327b04c5b980b8c52ffe91f535eb47981a17793fac77b4448e82fb69e7a241735bf5288d70c0822968914a6d9aa31226d6a1317711d88f36ac860004055f4118946c7e1d5bcaf8b0a2e4d6e15c228abf4204ded888cafad419c4576554aec0482bb6c6658ac1a91c51ac42e032f9b961f2d55de62258df1841004681f",
    "010000008dfd039b71b2277a3b808c9602f53588dc8b596bfd618cf48fdb9801bfc0b60c6ef8bcba52afc2c668634f27ae0d7cbf8004b52ba53657aeb13b14efaac613a307c23130728a829a2bfe0156ab69e3efbff95c10800833c239d68b235a46a7c3f36ac8600040704a331e98725dc1ad5c31e6227300834da90ab5e14d25d97ef0a5e44e2d3cbe84b37014aa42a2d1d688fcf199ac511d3d87bd181b49eb4afc1ca55ca06ebcb4",
    "0100000097667d09b34d37324f5bdf93b21542042f81527adf631bdadf07e7519ff85a4237ed402d1c2a35f449d230ae044a7b9e05a869345c4c20a1d20f4b051da74cc3ffe3f5c2d4108cdcdfa419a155aa6c66ad9317543112f6723043f57f44db6d22f36ac86000404f7a0da81f21b6a156ca887c9607ff957f11345d02a51b77952ecb80dc70f001eb524b547e95f391c7ad43d1f5cc3c7aa4df88c86a37230b1a2241f7f1789401",
    "010000004c04399385f4ccf9b67ce33686a52daea86ffadedf0f13bb3c031b99e862a5d3152b16c7ae81844ab9a711e6d6bed4bf3bcdc9e23940a202c3563be39a467e91c49e571f00cddb7118f22090875ba3eaad5e4ff8bfca14ac49ba0ba246ba7e1bf46ac860004030c0a392206cdc180b99afa70a09c69f96bb258f0ace5d90a5fbace751ca08c46d8758db3b2d3308b28e3fdae34340b9451a3770e521203c42551c86a11d9885",
    "01000000c00e65f0d9a3f19a737272b580d809ecd746f465636d35fa38f0e4797ab8fe009b5376e6b513f7b79e59febc2ae8e991806e6632b41552092c6572522b8628a4cc474090aca162fec3a26f59b20d88e7ab458a3c9397e980e9e90819bef75842f46ac8600040e87efeb69d585ea79c2e78a2b7887a12ff6b73a9063c3b67b17c5e536e2709b9709d2aafe5a2de3a520ab779699cb3f19519b34a4abf5c5971911609be45f996",
    "0100000025cb7c9e0e80adf69ef00647f9dbd0b841943f683435e156754fd52dff9feceb41aec69ccc502b1651c6227cd934c2b3840ceddf1fce1571184cdb47aa5f75671bab4e26f868816bc749654352f6ba30ec092ae7657735e2ebe388752ea70b76f46ac860004080b56b950cdbd5eab9109c4ea5c4fa87eadabbde68c7875b910265bd4c3b62c5df11932fd3f893c0f3f4909850a459769b25848b75d55a7bf34fd5d28b6520ca",
    "0100000018170316332e315f95dcc07dfc7cab0e0f4a787f9e18f8bccc74a704be38afc9182a9ce23787690b159cc3916fe72aa3de6ec5bf2fe784dfe780d9452431ff8cda45548bc4f8868adf32d2fb14a390b4961acce2d6baa72d9cf62a2eebd652def46ac860004059885b57d6590a2eeb2f64889500f86d2239023a83c6106ed4a4729701ec97b011c2815aa0569e037907ea1d4cae9b840f4929cd8c368f80442016212ae116f4",
    "010000008df6ae8f2fca55037d05f13182b5b84cda31b85e2b0cadf576e3b7463d1f9fcfb76c5ee3687538d8defb36eeeed7e58902a08570b2eb97646b43b4a2088057db1254793164268c7d108625f64b0a1077080b7dde11f447f3157b96b45dd6b339f46ac86000404eb23cd4d608f870c7144850ed26aa288ad577abf31e222993006b20dee967f89b42e2c5b9c56c2845859fe6e169b650e48737023ea964f2b0330c726f7074f3",
    "010000009a150cf4747cc3bd502d815292edb8a2e81a187d5d727e81c1ce067799305652f2e7ba89ef051cd8af194386c363f39c62b7d0bf450de8b00b123f1dcbeb88bce07dd7891f4f2a11032145f017d16c1ea3d384afe926d23842a8b944a2f687aef46ac86000404ef5e3bc5d01f8de7bfedbd66427b8315889a7fccdd95cb3b0c7f1941ce5393e588306a83f5a6014330d8cd6c13f6d1280f307dd3a2afdb2e39f0f56fd05b7b9",
    "01000000c75a4a64ce47660b041b84f4a75b6b94c8a6956ed97295343fc4d0dbc05afb411bd8be4454525a55247ddea9feae7edfa34249d9644ab4242915be4f0ddb8aabf57f8e6dd372cead297fae68c7ba6629e4b62e3b43b6566c53399269cd46c398f56ac8600040dbd90b1ea2af4d554cb9e998f1519ca4564051f9cb31d24d067af6d609a51b693e9a8d603a0a58813d4e3d42d44f49ca94af8554d9a6e1c1803601bf0f60862e",
    "01000000f55370b33e74a24b2a5f34d51d06496e2add6590dc67479b40f98dc96ca1e082ecebbaae62be783818ea6f8547742760027b70e4f25d9f9f61ac16a316a5c451ef03432e2c71962a9a5a037471b98a8cb3b573bbae471ed47f9d75623f264baef56ac8600040097558668fddaf85184b6e15455784a9825b1c08537ae878fced1132cb451b1fe5dea4c3002bf8b85d8e516257e62b8ad11543bcf625f917e6fca157dd6ce279",
    "01000000d17ae7e0e0ce9d8cfc8fdf554c76b0ff9ce0e440573ea1327dbcea7be27a331a26813cf6b083bb81908f4142938c0172d41f6808617614dfba537906f7e7065793fa268a10a06989ca80e11cb60859cb2d934dbc42c6981931b4a7db37be75e1f56ac860004088a1d005daf9ce30790afe24fdf0daa384c9ffca60a4e3f273e2a70be4ac3df6f92f9b84a0f01d452f42c78b3978a2e895a6171b42df95a0a7ef7c4ff14d6868",
    "01000000c94dfdbc465eb8d28eae1c468c2b7912c8250b43e1867b983c3fa7f992d1acddd40e74a4065e8c090e76ec8b87675308665a50c1c92524838e989d852fd3a840e7e0d32f406b8d5fb76445061edb55b6f2b55ab96f723eaba11f196822988c5bf56ac8600040d9b7d8b1d02a8e4b1286972b5a52f86fb3fcbb339219e4231f87854fb9cb9a3aa4a7908bbae3630ca71dc580d52e6382fef4f1615555c57502031df7f6a2ad17",
    "010000008e207678f06e5768e8aa98199539770e3bacbe6a46946fe8f7c05159146453cdbb677df895c799b3acb62ab65ba2743114d9b2c387919e674b5c26acc35244e573b1bda8a13cd41aee359cff3d0da51f2ed29914d75074e9f802f34dd8028293f56ac86000407b580b77c8ec71be17692161d24f0eb40e1c8296b880cc9094ac15bc2a83a5d037b382d07f8cf49059a516e76b24014b6a487cd738ee4a1ac61f68df8758cd19",
    "0100000060fc7b4f817c09be85724d91fcd6d2355ac7807f9918cdcff5845ad205548c11050968c4f1874090c1c3f8354edded34e3e4ae9f4e202bae14f1ba4135b7c5ab24e07882852adc96fe029cd78d68aef1bc1f5cda7ec5222a8ec6e2ea65206bdff56ac8600040f388fd73d09ad1c170a9701b609005de225f88ebd57d7e5cb1cd53552f9eb2334dd75fa50141f68b146fcf507138155daaaff40458554a37359714f3df93d47e",
    "01000000fb07788543f811803c8307adad63592965eeb63b1a57eac45aade6a1aadf81a9ef5068898a38fb9bd1ef8a31e82810ea2e0c36238543a08ee04bfd55dbecd1a3389eba298c54c45c70d441b834373e61def5f14e30ea596f6c47cbb50b385345f66ac860004069be80e5a24dbd7e1266f37e796621ddb9df0855a1059989714776629f4a1d37b89930d7ab9312a5bb7948eecc8c39c86c08197303a0c0b108cc146be440e65e",
    "01000000dd101581ecbf3d07e76f78dc5ea4b08212e4f12b9663da414f90f2cc3dc07cd9ed5a0f63eb226e22b0b7a16c856b216ecc6b5f9213d5d0333d180e8447f7117ce15cfb8a6b7f2c8381fefcc108b768a096ad95a90e7cf4dcec234329003ead92f66ac8600040a4828842336ca3d15f923f8f73648a81e1d71eec1ad8865287dc65b1e47e19edd93927a56dc7ed7e3ad9d713e7109bed457f4901ab2a5d9dbf1aab00e549c167",
    "0100000013c09275a3e68fc743bc5ccddcf867ecf9e654c270be8f098e804360434a36872ec20866a5337af0d2d1e9cd0525e60169f95b230a1d9af971b3a5d8b69da9f194bd696a7dedf95968c14d12fdf2c3b729bf3d990c08e0bbeea90e7035aeafddf66ac8600040a1ff6b6fb5598d61c303e2bf5335d757813a087f274a2fcf2c78cfcdf82b7da754e8709f14d1c3cad8b3d4a2245b519b6d4b12e7242cefe793a1e9bfb75072fd",
    "01000000c49e22abad9a2098f1f7de189f390c6270603d77efe2d7f888519f8d1979d472cc2095663d2c1cea2e0fcc6378863dc4e97b18828d0692ad995e04aff325f854ef1ef2b69490abe13ee54469045b3f90041bb7de5524a3d69def74c1d9ad2a2cf66ac8600040e5fde505618033225490c327aba0d04709425bb971b71a7cdad9228b0bc18ed95236d4799a305d5b8bbed183f60a57dc0d8a537a20cde1f94a052327085af738",
    "0100000013533800e5eb88bea521a97c8c2d0fd66e00d3cc86cbd5ba5a038a36cd14844c8ff8bd13c91f754d25cd574d6aaef2885b79732e52a3963b39f8a51308dffa7f4dc87a75c0a791bad8719178d271d4953367d948790c54507953121fecebc1eaf66ac86000402bec2ef49a257c3b94c87a498f3ffcea1d342d1bf60e49dce4a69a33b4b324a90a9e8ca29de17309449fd0a5e4385597beab46ce9c9a50b1261c3013cb73fca4",
    "0100000036b907618eb6aa14b1fb3c9bca1dd89d8b4954d19894c61852dc4a8a3f54d978c358633ce8224ac0ff3d1a9c8ff59a11e2a28b222aa2777baaabda17d91bf560c13bccca1a531674bfff5c4270c28cc9b2a79fa500ab52e996b9b5b0d9557fe4f66ac86000405197698e938130f5fa9f49d30c12d703b15a6697c882c0d9b0bf45501f7d2e7a75595053bc4bd2aa682097d7a7957b736ab0e138aef88f9f81863274c2dc8332",
    "010000003d839c2988a8b67fd3d880374b6a06183a37846049979bf6b756dcf7d5dbb9f639f588562df3ca621ef33ea27830f20be9b3aaafa8c29d68fe9a532720864f00570730ac1b21f503fc55a20a25cfdfe8435577b05f460334fee37d5f39784034f76ac86000401abbf1f965d75973631fe6040a99fcb4f77e043d1fcb8468389c975324e12148316bfc5722ca84f20b5668a463925111500477f7f86fe8880a391ada69ee32ca",
    "01000000ac6017ade9a79940e5380d43fff8bdf713e81bbd9fdb691144c2e82ffe7f49fc41ac5269352c4f285d893943de4c95666a31ff6bf2802350480f088e1bc94aadc706f028137081e7266f723638d08e82ebcb2be2b3ac1c2d423dff5152e34a38f76ac860004060201a3c0e0afd2a75f1a093068366a506077cf9420405f7ad0e49c71a47f0b6965040c060f2fac9c87a0c70440d1c0a8b91b0ff11cefc1cf88a62e76c11bc63"
];

pub static GENESIS_BLOCK_HEX: &str = "010000000000000000000000000000000000000000000000000000000000000000000000623fd6e71aaec98e129d8b447ba7c6fe88cd27346cc556353d2d8232a2829f0a49b4a19f4dc3f0526dca905dcaff6a8e34537d04b450e0ac5568ce89a9373e301665c860012102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a40f457d5dd7caae6bf89a50efd13cf4a9e3857760f747e107d1184263e1212ef98cda52410bd822e92c44b4a22a2f6116a0df2b130afe315af6a7289567b32c1ca01010000000100000000000000000000000000000000000000000000000000000000000000000000000000ffffffff0100f2052a010000002776a9226d6b597162714c54584e52344568747853376e37734e5357385646546f314e55336e88ac00000000";

pub fn get_test_genesis_block() -> Block {
    let bytes = hex_decode(GENESIS_BLOCK_HEX).unwrap();
    deserialize(&bytes).unwrap()
}

pub fn get_test_block_hash(height: usize) -> BlockHash {
    get_test_headers(height, 1).first().unwrap().block_hash()
}

pub fn get_test_block_index(height: i32) -> BlockIndex {
    let header = get_test_headers(height as usize, 1).pop().unwrap();
    BlockIndex {
        header,
        height,
        next_blockhash: BlockHash::default(),
    }
}

pub fn get_test_headers(start: usize, count: usize) -> Vec<BlockHeader> {
    let mut result: Vec<BlockHeader> = vec![];

    for hex in &HEADER_STRINGS[start..start + count] {
        let bytes = hex_decode(hex).unwrap();
        let header = deserialize(&bytes).unwrap();
        result.push(header);
    }

    result
}

// return initialized chain
pub fn get_chain() -> Chain<OnMemoryChainStore> {
    let mut store = OnMemoryChainStore::new();
    store.initialize(get_test_genesis_block());
    Chain::new(store)
}

pub struct TwoWayChannel<T> {
    sender: UnboundedSender<T>,
    receiver: UnboundedReceiver<T>,
}

pub fn channel<T>() -> (TwoWayChannel<T>, TwoWayChannel<T>) {
    let (sender_in_here, receiver_in_there) = tokio::sync::mpsc::unbounded_channel::<T>();
    let (sender_in_there, receiver_in_here) = tokio::sync::mpsc::unbounded_channel::<T>();

    let here = TwoWayChannel::new(sender_in_here, receiver_in_here);
    let there = TwoWayChannel::new(sender_in_there, receiver_in_there);

    (here, there)
}

impl<T> TwoWayChannel<T> {
    pub fn new(sender: UnboundedSender<T>, receiver: UnboundedReceiver<T>) -> TwoWayChannel<T> {
        TwoWayChannel { sender, receiver }
    }
}

impl<T> Sink for TwoWayChannel<T> {
    type SinkItem = T;
    type SinkError = Error;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.sender
            .start_send(item)
            .map_err(|e| Self::SinkError::from(e))
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sender
            .poll_complete()
            .map_err(|e| Self::SinkError::from(e))
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sender.close().map_err(|e| Self::SinkError::from(e))
    }
}

impl<T> Stream for TwoWayChannel<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.receiver.poll().map_err(|e| Self::Error::from(e))
    }
}
