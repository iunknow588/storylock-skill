# StoryLock Android 瀹夸富瀹炵幇瑙勮寖

## 1. 鐩爣

鏈枃妗ｇ敤浜庢妸褰撳墠 `Android 瀹夸富 mock` 鏀舵暃涓哄悗缁湡瀹?Android 瀹夸富瀹炵幇鐨勬渶灏忚鑼冦€?
褰撳墠鐩爣涓嶆槸瀹氫箟瀹屾暣 App 浜у搧锛岃€屾槸瀹氫箟鈥滅涓€灞傚拰绗簩灞傚浣曞湪 Android 渚ф壙杞斤紝骞朵笌绗笁灞備簯绔叆鍙ｅ鎺モ€濄€?
## 2. 褰撳墠鏋舵瀯瀹氫綅

寤鸿閲囩敤浠ヤ笅閮ㄧ讲鍏崇郴锛?
1. 绗笁灞?`storylock-remote-gateway-skill` 閫氳繃 Vercel 椋庢牸鍏ュ彛瀵瑰鏆撮湶銆?2. 绗竴灞?`storylock-local-story-processing-skill` 鍦?Android 鏈湴鎵ц銆?3. 绗簩灞?`storylock-local-story-access-skill` 鍦?Android 鏈湴鎵ц銆?4. Android 瀹夸富瀵圭涓夊眰鍙毚闇叉渶灏忔帴鍙ｏ細`GET /health`銆乣POST /execute`銆?
## 3. Android 瀹夸富蹇呴』鎵胯浇鐨勮兘鍔?
### 3.1 绗竴灞?
鑷冲皯闇€瑕侊細

1. 棰橀泦寮哄害妫€鏌ャ€?2. 鏈湴鏁呬簨棰橀泦鍑嗗鐘舵€佹鏌ャ€?
瀵瑰簲褰撳墠浠ｇ爜鑳藉姏锛?
1. `StrengthReviewSkill`

### 3.2 绗簩灞?
鑷冲皯闇€瑕侊細

1. 瀵硅薄寮哄害绛栫暐鍒ゅ畾銆?2. 涔濆鏍?challenge 鍒涘缓銆?3. 鏈湴绛旀鏍￠獙銆?4. 鐭椂鎺堟潈绛惧彂銆?5. 闃查噸鏀俱€佸け璐ラ攣瀹氥€佸璁¤惤搴撱€?6. 鎾ら攢鎺ュ彛棰勭暀銆?
瀵瑰簲褰撳墠浠ｇ爜鑳藉姏锛?
1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`
4. `LocalRevocationSkill`

## 4. Android 瀹夸富鏈€灏忔帴鍙?
### 4.1 `GET /health`

鐢ㄩ€旓細

1. 缁欑涓夊眰妫€鏌?Android 瀹夸富鏄惁鍦ㄧ嚎銆?2. 杩斿洖绗竴灞傞闆嗗氨缁姸鎬併€?3. 杩斿洖绗簩灞?active question set 鐨勬瑕佺姸鎬併€?
褰撳墠 schema锛?
`src/skills/remote-gateway/assets/schemas/android-host-health.schema.json`

### 4.2 `POST /execute`

鐢ㄩ€旓細

1. 鎺ユ敹绗笁灞傛爣鍑嗚姹傘€?2. 鍦?Android 鏈湴瀹屾垚鏈湴鎺堟潈閾捐矾銆?3. 杩斿洖鏍囧噯 remote gateway response銆?
褰撳墠杈撳叆杈撳嚭 schema锛?
1. `src/skills/remote-gateway/assets/schemas/remote-gateway-request.schema.json`
2. `src/skills/remote-gateway/assets/schemas/remote-gateway-response.schema.json`

## 5. Android 渚ф湰鍦板畨鍏ㄨ姹?
### 5.1 瀵嗛挜涓庨暱鏈熺瀵?
鐪熷疄 Android 瀹夸富搴斾紭鍏堟帴鍏ワ細

1. Android Keystore
2. 蹇呰鏃剁粨鍚?EncryptedSharedPreferences 鎴栫瓑浠峰畨鍏ㄥ瓨鍌ㄤ繚瀛橀潪瀵嗛挜鍏冩暟鎹?
褰撳墠蹇呴』婊¤冻锛?
1. 涓嶆妸闀挎湡绉侀挜鏄庢枃鍐欏叆鏅€氭枃浠躲€?2. 涓嶆妸 challenge answers 鏄庢枃闀挎湡鎸佷箙鍖栥€?3. 涓嶆妸 masterSalt 鏄庢枃鏆撮湶缁欑涓夊眰鎴栬繙绔€?
### 5.2 challenge answers

鐪熷疄 Android 瀹夸富蹇呴』閬靛畧褰撳墠鎸戞垬绛旀瀛樺偍绛栫暐锛?
1. 鍘熷绛旀鍙厑璁哥煭鏃跺瓨鍦ㄤ簬鍐呭瓨鎴栧彈鎺ц緭鍏ユ祦绋嬨€?2. 鎸佷箙灞傚彧淇濆瓨鎽樿銆乧hallenge manifest 鍜?session 鍏冩暟鎹€?3. challenge 瀹屾垚鍚庣珛鍗虫竻鐞嗚繍琛屾€佸師濮嬬瓟妗堛€?
### 5.3 鐢熺墿璇嗗埆涓庢湰鍦扮‘璁?
寤鸿鐪熷疄 Android 瀹夸富澧炲姞浠ヤ笅鏈湴纭鑳藉姏锛?
1. BiometricPrompt 浜屾纭銆?2. 搴旂敤鍓嶅悗鍙板垏鎹㈡椂鑷姩澶辨晥寰呮巿鏉冪姸鎬併€?3. 楂樺己搴︾鍚嶈姹傚彲瑕佹眰鈥滈闆嗛獙璇?+ 鐢熺墿璇嗗埆鈥濆弻鏉′欢銆?
娉ㄦ剰锛? 
鐢熺墿璇嗗埆鏄?Android 瀹夸富鐨勬湰鍦板姞鍥烘満鍒讹紝涓嶅簲鏇夸唬棰橀泦 challenge 鐨勪富濂戠害锛岄櫎闈炴湭鏉ユ枃妗ｅ彟琛屽崌绾у畾涔夈€?
## 6. 绗笁灞備笌 Android 瀹夸富鐨勮繛鎺ヨ姹?
### 6.1 绗笁灞備簯绔叆鍙?
褰撳墠绗笁灞傚叆鍙ｏ細

1. `web-api/storylock-gateway.mjs`
2. `src/skills/remote-gateway/web-api-handler.js`
3. `GET /download/android-host`

绗笁灞傝亴璐ｏ細

1. 鎺ユ敹杩滅▼璇锋眰銆?2. 淇濇寔涓绘帴鍙?allowlist銆?3. 杞彂鏍囧噯 envelope 缁?Android 瀹夸富銆?4. 瀵硅繑鍥炵粨鏋滃仛鑴辨晱銆?5. 鍚戝鎻愪緵 Android 鏈湴瀹夸富鐨勪笅杞藉湴鍧€涓庣浜屽眰杩炴帴鍏冩暟鎹€?
### 6.2 瀹夸富閴存潈

褰撳墠鏈€灏忔柟妗堬細

1. `x-storylock-shared-secret`

鍚庣画鐪熷疄瀹炵幇寤鸿澧炲姞锛?
1. 璁惧鏍囪瘑 `deviceId`
2. App 瀹炰緥鏍囪瘑 `appInstanceId`
3. 瀹夸富娉ㄥ唽璁板綍
4. 鍙€夎澶囪瘉鏄庢垨绛惧悕鎸戞垬

## 7. 褰撳墠浠撳簱宸茬粡楠岃瘉鐨勯儴鍒?
褰撳墠浠撳簱宸茬粡楠岃瘉锛?
1. 绗笁灞?Vercel 椋庢牸鍏ュ彛鍙繍琛屻€?2. Android 瀹夸富 mock 鍙壙杞界涓€灞備笌绗簩灞傝兘鍔涖€?3. `selftest:web-api-android` 鑳借窇閫氱鍒扮璇锋眰閾捐矾銆?
褰撳墠浠撳簱灏氭湭鎻愪緵锛?
1. 瀹屾暣 Android App 宸ョ▼銆?2. Android Keystore 瀹炵幇銆?3. BiometricPrompt 闆嗘垚銆?4. 鐪熸満缃戠粶鏆撮湶鏂规銆?
## 8. 鎺ㄨ崘瀵瑰鍙ｅ緞

寤鸿浣跨敤锛?
> 褰撳墠浠撳簱宸茬粡楠岃瘉鈥滅涓夊眰浜戠鍏ュ彛 + Android 鏈湴瀹夸富鈥濈殑鏈€灏忛摼璺紝鍏朵腑绗笁灞傝礋璐ｈ姹傚寘瑁呬笌鑴辨晱锛岀涓€灞傚拰绗簩灞備繚鐣欏湪鏈湴瀹夸富锛涚湡瀹?Android App 涓?Android Keystore 闆嗘垚灞炰簬涓嬩竴闃舵瀹炵幇宸ヤ綔銆?

