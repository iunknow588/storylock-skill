# StoryLock 濞村鐦弬瑙勵攳 v1.0

閻楀牊婀伴敍姝?.0  
閺冦儲婀￠敍?026-06-17  
闁倻鏁ら懠鍐ㄦ纯閿涙瓪E:\2026OPC婢堆嗙\skill\src` 瑜版挸澧犳禒锝囩垳閸╄櫣鍤? 
濞村鐦弬瑙勵攳娴ｅ秶鐤嗛敍姝欵:\2026OPC婢堆嗙\skill\docs\test\StoryLock濞村鐦弬瑙勵攳_v1.0.md`

## 娑撯偓閵嗕焦绁寸拠鏇犳窗閺?
閺堫剚绁寸拠鏇熸煙濡楀牏鏁ゆ禍搴ㄧ崣鐠?StoryLock 瑜版挸澧犳稉澶婄湴 Skill 閺嬭埖鐎弰顖氭儊閹稿顔曠拋鈥充紣娴ｆ粣绱?
1. 缁楊兛绔寸仦鍌濆厴鐎瑰本鍨氶弫鍛皑閼藉顭堥妴浣规櫊娴滃榧庨懝鎻掓嫲妫版﹢娉﹀鍝勫鐠囧嫪鍙婇妴?2. 缁楊兛绨╃仦鍌濆厴鐎瑰本鍨氱€电钖勫鍝勫閸掋倖鏌囬妴浣风瘈鐎诡偅鐗告宀冪槈閵嗕胶鐓弮鑸垫拱閸︾増宸块弶鍐︹偓渚€妲婚柌宥嗘杹閵嗕礁銇戠拹銉╂敚鐎规艾鎷?SQLite 鐎孤ゎ吀閵?3. 缁楊兛绗佺仦鍌濆厴鐎瑰本鍨氭潻婊呪柤缁涙儳鎮曠拠閿嬬湴閵嗕箘eb2 鐎靛棛鐖滄繅顐㈠帠鐠囬攱鐪伴妴涓扞P-712 缂佹挻鐎崠鍛邦棅閵嗕焦婀伴崷鐗堝⒔鐞涘苯娅掓慨鏃€澧崪宀勨偓鎺戠秺閼磋鲸鏅遍妴?4. 閸忕厧顔愬鏃傘仛閸栧懓鍏樻穱婵囧瘮閸╄櫣顢呴崣顖濈箥鐞涘矉绱濇稉宥勭稊娑撶儤鏌婇惃鍕瘜缁惧灝鐣ㄩ崗銊ョ湴閵?5. 閺傚洦銆傞妴涓糲hema閵嗕線鏁婄拠顖滅垳閸滃苯缍嬮崜宥勫敩閻椒绻氶幐浣风閼锋番鈧?
Deprecated historical interfaces only; no longer in current mainline:
閺堫剚鏌熷鍫滅瑝閸愬秵绁寸拠鏇熸＋閹恒儱褰?`requestStoryRead`閵嗕梗requestStoryWrite`閵嗕梗requestChallengeSign`閵嗕梗StoryReadAccessSkill`閵嗕梗StoryWriteAccessSkill`閵嗗倽绻栨禍娑欏复閸欙絼绗夌仦鐐扮艾瑜版挸澧犳稉鑽ゅ殠閵?
## 娴滃被鈧焦绁寸拠鏇″瘱閸?
| 濞村鐦€电钖?| 鐠侯垰绶?| 娴兼ê鍘涚痪?| 鐠囧瓨妲?|
| --- | --- | --- | --- |
| 缁楊兛绔寸仦鍌涙櫊娴滃顦╅悶鍡楀瘶 | `src/skills/local-story-processing` | P0 | 閼藉顭堥妴浣归紟閼瑰眰鈧礁宸辨惔锕佺槑娴?|
| 缁楊兛绨╃仦鍌涙拱閸︽媽顔栭梻顔藉房閺夊啫瀵?| `src/skills/local-story-access` | P0 | 鐎电钖勫鍝勫閵嗕椒绡€鐎诡偅鐗搁妴浣规拱閸︾増宸块弶鍐︹偓涓糛Lite 閻樿埖鈧?|
| 缁楊兛绗佺仦鍌濈箼缁嬪缍夐崗鍐插瘶 | `src/skills/remote-gateway` | P0 | `requestSignature`閵嗕梗requestPasswordFill`閵嗕浇鍔氶弫?|
| 閺勬挸鐣ㄧ純鎴犵彲娑撳海顑囨稉澶婄湴闁劎璁查崗銉ュ經 | `src/ui`閵嗕梗web-api/storylock-gateway.mjs` | P0 | 妫ｆ牠銆夐妴浣稿蓟鐠囶厼鍨忛幑銏″付娴犺翰鈧胶濮搁幀浣瑰复閸欙絻鈧竸PK 娑撳娴囨稉搴ｇ拨鐎规艾鍙嗛崣?|
| 閸忕厧顔愬鏃傘仛閸?| `src/engine` | P1 | 閺堫剙婀寸€靛棛鐖滄繅顐㈠帠閸滃瞼顒烽崥宥嗗房閺夊啰銇氭笟?|
| 鐠恒劌瀵橀梿鍡樺灇闁炬崘鐭?| 缁楊兛绗佺仦?-> 缁楊兛绨╃仦?-> 閺堫剙婀撮幍褑顢戦崳?| P0 | 缁旑垰鍩岀粩顖涘房閺夊啯澧界悰?|
| Schema 娑撳孩鏋冨锝咁殩缁?| `assets/schemas`閵嗕梗docs/design/cn` | P1 | 鐎涙顔岄妴渚€鏁婄拠顖滅垳閵嗕浇鍏橀崝娑樻倳娑撯偓閼?|

## 娑撳鈧焦绁寸拠鏇犲箚婢?
| 妞ゅ湱娲?| 鐟曚焦鐪?|
| --- | --- |
| 閹垮秳缍旂化鑽ょ埠 | Windows 10/11閿涘苯鍚嬬€?Linux/macOS |
| Node.js | 22.0.0 閹存牔浜掓稉?|
| npm | 闂?Node.js 22 鐎瑰顥婇悧鍫熸拱閸楀啿褰?|
| SQLite | 娴ｈ法鏁?Node.js `node:sqlite` |
| 濞村鐦弫鐗堝祦鎼?| 姒涙顓绘担璺ㄦ暏娑撳瓨妞?SQLite 閺傚洣娆㈤幋?`:memory:` |
| SecretStore | 閸楁洖鍘撻崪宀冨殰濞村濞囬悽?`MemorySecretStore`閿涘本瀵旀稊鍛濞村鐦箛鍛淬€忛弰鎯х础濞夈劌鍙?secretStore |

閻滎垰顣ㄥΛ鈧弻銉ユ嚒娴犮倧绱?
```powershell
node -v
npm -v
```

瑜版挸澧犻張鈧亸蹇涚崣鐠囦礁鎳℃禒銈忕窗

```powershell
Push-Location E:\2026OPC婢堆嗙\skill
npm run test
Pop-Location
```

娑旂喎褰叉禒銉ュ瀻閸掝偉绻嶇悰宀嬬窗

```powershell
Push-Location E:\2026OPC婢堆嗙\skill\src\storylock-local-story-processing-skill; npm run selftest; Pop-Location
Push-Location E:\2026OPC婢堆嗙\skill\src\storylock-local-story-access-skill; npm run selftest; Pop-Location
Push-Location E:\2026OPC婢堆嗙\skill\src\storylock-remote-gateway-skill; npm run selftest; Pop-Location
Push-Location E:\2026OPC婢堆嗙\skill\src\storylock-skill-engine; npm run selftest; Pop-Location
```

## 閸ユ稏鈧焦绁寸拠鏇炲瀻鐏炲倻鐡ラ悾?
| 濞村鐦猾璇茬€?| 閻╊喗鐖?| 瀵ら缚顔呭銉ュ徔 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- |
| 閸愭帞鍎ù瀣槸 | 绾喛顓婚崶娑楅嚋閸栧懎缍嬮崜宥堝殰濞村褰叉潻鎰攽 | 閻滅増婀?`npm run selftest` | P0 |
| 閸楁洖鍘撳ù瀣槸 | 妤犲矁鐦夐崡鏇氶嚋 Skill 閹存牕鍤遍弫鎷岊攽娑?| Node.js Test Runner | P0 |
| 闂嗗棙鍨氬ù瀣槸 | 妤犲矁鐦夌捄銊ュ瘶闁炬崘鐭?| Node.js Test Runner + 娑撳瓨妞?SQLite | P0 |
| 鐎瑰鍙忓ù瀣槸 | 妤犲矁鐦夐梼鏌ュ櫢閺€淇扁偓渚€鏀ｇ€规哎鈧浇鍔氶弫蹇嬧偓浣规櫛閹扮喎鐡у▓鍏哥箽閹?| 閼奉亜鐣炬稊澶屾暏娓?| P0 |
| 婵傛垹瀹冲ù瀣槸 | 妤犲矁鐦?Schema閵嗕線鏁婄拠顖滅垳閵嗕焦鏋冨锝呭經瀵板嫪绔撮懛?| Ajv + 闂堟瑦鈧焦澹傞幓?| P1 |
| 閸忕厧顔愬ù瀣槸 | 妤犲矁鐦?`storylock-skill-engine` 閸欘垵绻嶇悰?| 閻滅増婀?selftest | P1 |

## 娴滄柣鈧礁缍嬮崜宥堝殰濞村鐔€缁?
瑜版挸澧犳禒锝囩垳瀹稿弶婀侀崶娑楅嚋閼奉亝绁撮崗銉ュ經閿?
| 閸?| 閸涙垝鎶?| 瑜版挸澧犵憰鍡欐磰闁插秶鍋?|
| --- | --- | --- |
| `storylock-local-story-processing-skill` | `npm run selftest` | 閼藉顭堥妴浣归紟閼瑰眰鈧礁宸辨惔锕佺槑娴艰埇鈧浇绔熼悾灞剧垼鐠?|
| `storylock-local-story-access-skill` | `npm run selftest` | 鐎电钖勫鍝勫閵嗕椒绡€鐎诡偅鐗搁妴渚€妲婚柌宥嗘杹閵嗕焦婀伴崷鐗堝房閺夊啨鈧礁銇戠拹銉╂敚鐎规哎鈧焦绔婚悶鍡愨偓涓糛Lite 鐎孤ゎ吀 |
| `storylock-remote-gateway-skill` | `npm run selftest` | `requestSignature`閵嗕梗requestPasswordFill`閵嗕笒IP-712閵嗕浇鍔氶弫蹇嬧偓浣规拱閸︾増澧界悰灞芥珤 |
| `storylock-skill-ui` | `npm run selftest` | 閺勬挸鐣ㄦ＃鏍€夐妴浣稿蓟鐠囶厼鍨忛幑銏″付娴犺翰鈧線娼ら幀浣界カ濠ф劑鈧胶缍夐崗宕囧Ц閹降鈧竸PK 閹芥顩︾€涙顔?|
| `storylock-skill-engine` | `npm run selftest` | 閺堫剙婀寸€靛棛鐖滄繅顐㈠帠閸滃瞼顒烽崥宥嗗房閺夊啫鍚嬬€瑰湱銇氭笟?|

P0 闁俺绻冮弽鍥у櫙閿涙艾娲撴稉?selftest 韫囧懘銆忛崗銊╁劥闁俺绻冮妴?
鐞涖儱鍘栫純鎴犵彲閼奉亝顥呴敍?
```powershell
Push-Location src/ui
npm run selftest
Pop-Location
```

鐠囥儴鍓奸張顒€鎯庨崝銊︽拱閸︾増妲楃€瑰缍夌粩娆忚嫙妤犲矁鐦夐敍?
1. 妫ｆ牠銆夐崣顖涘ⅵ瀵偓閵?2. 娑擃叀瀚抽弬鍥у瀼閹广垺甯舵禒璺虹摠閸︻煉绱濋崜宥囶伂閸栧懎鎯堟稉顓熸瀮娑撳氦瀚抽弬鍥ㄦ瀮濡楀牆鐡ч崗鎼炩偓?3. 闂堟瑦鈧浇绁┃鎰讲鐠囪褰囬妴?4. `GET /api/storylock-gateway` 閸欘垵绻戦崶鐐靛Ц閹?JSON閵?5. APK 閻楀牊婀伴妴浣圭墡妤犲苯鈧棿绗岄崠鍛閸ㄥ鐡戦幗妯款洣鐎涙顔岄崣顖欑矤鏉╂劘顢戦幀浣哄Ц閹浇顕伴崣鏍モ偓?
## 閸忣厹鈧胶顑囨稉鈧仦鍌涚ゴ鐠囨洜鏁ゆ笟?
濞村鐦€电钖勯敍?
1. `StoryDraftSkill`
2. `StoryRefineSkill`
3. `StrengthReviewSkill`

### 6.1 StoryDraftSkill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| P-DRAFT-001 | 濮濓絽鐖堕悽鐔稿灇閼藉顭?| `objective`閵嗕梗audience`閵嗕梗tone`閵嗕梗constraints` | 鏉╂柨娲?`mode=story_draft`閿涘苯瀵橀崥?`draft` | P0 |
| P-DRAFT-002 | 缂傚搫銇?objective | 娑撳秳绱?`objective` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-DRAFT-003 | 缁屽搫鐡х粭锔胯 objective | `objective=""` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-DRAFT-004 | 姒涙顓?audience/tone | 閸欘亙绱?`objective` | 娴ｈ法鏁ゆ妯款吇 `self`閵嗕梗neutral` | P1 |
| P-DRAFT-005 | 閼奉亜鐣炬稊?generator | 濞夈劌鍙?generator | 鏉╂柨娲栭懛顏勭暰娑斿宕忕粙鍖＄礉娑撴梻绮ㄩ弸鍕潶閺嶏繝鐛?| P1 |
| P-DRAFT-006 | 闂堢偞纭?source | `source=remote_raw_secret` | 閹舵稑鍤?source 閺嬫矮濡囬柨娆掝嚖 | P0 |
| P-DRAFT-007 | 鏉堝湱鏅弽鍥唶 | 閺堝鏅ユ潏鎾冲弳 | `challengeCreated=false`閵嗕梗sessionIssued=false`閵嗕梗protectedObjectRead=false` | P0 |

### 6.2 StoryRefineSkill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| P-REFINE-001 | 濮濓絽鐖跺☉锕佸 | `storyDraft`閵嗕梗goals`閵嗕梗hintStyle` | 鏉╂柨娲?`mode=story_refine` 娑?`refinedDraft` | P0 |
| P-REFINE-002 | 缂傚搫銇?storyDraft | 娑撳秳绱?`storyDraft` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-REFINE-003 | storyDraft.content 娑撹櫣鈹?| `content=""` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-REFINE-004 | 闂堢偞纭?source | `source=template_only` | 閹舵稑鍤?source 閺嬫矮濡囬柨娆掝嚖 | P0 |
| P-REFINE-005 | 閼奉亜鐣炬稊?refiner | 濞夈劌鍙?refiner | 鏉╂柨娲栭懛顏勭暰娑斿榧庨懝鑼波閺?| P1 |
| P-REFINE-006 | 鏉堝湱鏅弽鍥唶 | 閺堝鏅ユ潏鎾冲弳 | 娑撳秴鍨卞?challenge閿涘奔绗夌粵鎯у絺 session | P0 |

### 6.3 StrengthReviewSkill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| P-STRENGTH-001 | 24 妫版ê宸辨０姗€娉?| 24 娑擃亪妫舵０姗堢礉濮ｅ繘顣?9 娑擃亜鈧瑩鈧銆?| `questionSetReady=true` | P0 |
| P-STRENGTH-002 | 妫版ɑ鏆熸稉宥堝喕 | 鐏忔垳绨?24 妫?| 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-STRENGTH-003 | 閸婃瑩鈧銆嶆稉宥嗘Ц 9 娑?| 閸楁洟顣介崐娆撯偓澶愩€嶆稉宥堝喕閹存牠鍣告径?| 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-STRENGTH-004 | 缂傚搫鐨張澶嬫櫏缁涙梹顢?| `validAnswers=[]` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| P-STRENGTH-005 | 瀵亶顣介梿鍡楃紦鐠?| 閸氬牐顫夋０妯圭瑝鐡?| 鏉╂柨娲?`issues` 閸?`recommendedActions` | P1 |
| P-STRENGTH-006 | 鏉堝湱鏅弽鍥唶 | 娴犵粯鍓伴張澶嬫櫏鏉堟挸鍙?| 娑撳秶顒烽崣?session閿涘奔绗夌拠璁崇箽閹躲倕顕挒?| P0 |

## 娑撳啨鈧胶顑囨禍灞界湴濞村鐦悽銊ょ伐

濞村鐦€电钖勯敍?
1. `ObjectStrengthPolicySkill`
2. `GridChallengeSkill`
3. `LocalAuthorizationSkill`
4. `access-host.js`
5. `sqlite-schema.sql`
6. `errors.js`

### 7.1 ObjectStrengthPolicySkill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| A-POLICY-001 | 缁涙儳鎮曠€电钖勬妯款吇妤傛ê宸辨惔?| `objectType=signature_key` 閹?`requestedAction=signature` | `requiredStrength=high`閿涘畭requiredCells=9` | P0 |
| A-POLICY-002 | 閸戭厽宓佺€电钖勬妯款吇娑擃厼宸辨惔?| `objectType=credential` 閹?`requestedAction=password_fill` | `requiredStrength=medium`閿涘畭requiredCells=6` | P0 |
| A-POLICY-003 | 閺咁噣鈧艾顕挒锟犵帛鐠併倓缍嗗鍝勫 | `objectType=generic_secret` | `requiredStrength=low`閿涘畭requiredCells=3` | P1 |
| A-POLICY-004 | policyHints 鐟曞棛娲婂鍝勫 | `policyHints.requiredStrength=high` | 娴ｈ法鏁ら幐鍥х暰瀵搫瀹?| P0 |
| A-POLICY-005 | 闂堢偞纭?objectType | `objectType=invalid` | 鏉╂柨娲?`SLG-001` | P0 |
| A-POLICY-006 | 闂堢偞纭?requestedAction | `requestedAction=invalid` | 鏉╂柨娲?`SLG-001` | P0 |
| A-POLICY-007 | 缂傚搫銇?identityId | 娑撳秳绱?`identityId` | 鏉╂柨娲?`SLG-001` | P0 |
| A-POLICY-008 | 缂傚搫銇?objectRef | 娑撳秳绱?`objectRef/credentialRef/keyId` | 鏉╂柨娲?`SLG-001` | P0 |

### 7.2 GridChallengeSkill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| A-GRID-001 | 娴ｅ骸宸辨惔锔跨瘈鐎诡偅鐗?| `requiredStrength=low` | 9 閺嶇厧鐫嶇粈鐚寸礉`requiredCells=3` | P0 |
| A-GRID-002 | 娑擃厼宸辨惔锔跨瘈鐎诡偅鐗?| `requiredStrength=medium` | 9 閺嶇厧鐫嶇粈鐚寸礉`requiredCells=6` | P0 |
| A-GRID-003 | 妤傛ê宸辨惔锔跨瘈鐎诡偅鐗?| `requiredStrength=high` | 9 閺嶇厧鐫嶇粈鐚寸礉`requiredCells=9` | P0 |
| A-GRID-004 | 缂冩垶鐗告稉宥堢箲閸ョ偟鐡熷?| 瀹?enroll 缁涙梹顢?| `grid.cells` 娑擃厺绗夐崠鍛儓 `answer` | P0 |
| A-GRID-005 | requestId 楠炲倻鐡戦柌宥嗘杹 | 鐎瑰苯鍙忛惄绋挎倱鐠囬攱鐪伴柌宥咁槻閹绘劒姘?| 缁楊兛绨╁▎陇绻戦崶鐐殿儑娑撯偓濞嗭紕绱︾€涙ê鎼锋惔?| P0 |
| A-GRID-006 | requestId 閸愯尙鐛?| 閻╃鎮?requestId閿涘奔绗夐崥?nonce 閹?payload | 鏉╂柨娲?`SLG-013` | P0 |
| A-GRID-007 | nonce 閸愯尙鐛?| 娑撳秴鎮?requestId閿涘瞼娴夐崥?nonce | 鏉╂柨娲?`SLG-013` | P0 |
| A-GRID-008 | 鏉╁洦婀＄拠閿嬬湴 | `expiry` 瀹歌尪绻?| 鏉╂柨娲?`SLG-011` | P0 |
| A-GRID-009 | 闂堢偞纭?requiredStrength | `requiredStrength=extreme` | 鏉╂柨娲?`SLG-001` | P0 |
| A-GRID-010 | 閺堫亝鏁為崘宀€鐡熷鍫熸喅鐟?| 閺?`enrollAnswers` | 鏉╂柨娲?`SLG-010` | P0 |
| A-GRID-011 | 鏉堟挸鍙嗛梹鍨闂勬劕鍩?| 鐡掑懘鏆?requestId/nonce/answer | 鏉╂柨娲?`SLG-001` | P1 |

### 7.3 LocalAuthorizationSkill

| 缂傛牕褰?| 閻劋绶?| 閸撳秶鐤嗛弶鈥叉 | 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- | --- |
| A-AUTH-001 | 濮濓絿鈥樼粵鏃€顢嶉幒鍫熸綀缁涙儳鎮?| 瀹告彃鍨卞?challenge閿涘苯鍑￠惂鏄忣唶缁涙梹顢?| `allowedAction=signature`閿涘本顒滅涵顔剧摕濡?| `approved=true`閿涘瞼顒烽崣?`authorizationId` | P0 |
| A-AUTH-002 | 濮濓絿鈥樼粵鏃€顢嶉幒鍫熸綀鐎靛棛鐖滄繅顐㈠帠 | 瀹告彃鍨卞?challenge閿涘苯鍑￠惂鏄忣唶缁涙梹顢?| `allowedAction=password_fill` | `readBudget=1`閿涘畭writeBudget=0` | P0 |
| A-AUTH-003 | 闁挎瑨顕ょ粵鏃€顢嶉幏鎺旂卜 | 瀹告彃鍨卞?challenge | 闁挎瑨顕ょ粵鏃€顢?| 鏉╂柨娲?`SLG-003` | P0 |
| A-AUTH-004 | 鏉╃偟鐢绘径杈Е闁夸礁鐣?| 閸?identity 鏉╃偟鐢婚柨?3 濞?| 缁?4 濞嗏€冲灡瀵ょ儤鍨ㄩ幓鎰唉 | 鏉╂柨娲?`SLG-004`閿涘苯鎯?`retryAfter` | P0 |
| A-AUTH-005 | 闁夸礁鐣剧粣妤€褰涢幁銏狀槻 | 闁夸礁鐣鹃弮鍫曟？瀹歌尪绻?| 閸愬秵顐奸崚娑樼紦 challenge | 閸忎浇顔忕紒褏鐢?| P1 |
| A-AUTH-006 | identity 娑撳秴灏柊?| identityB 閹绘劒姘?identityA 閻?challenge | 鏉╂柨娲?`SLG-003` | P0 |
| A-AUTH-007 | challenge 闁插秴顦查幓鎰唉 | 閸氬奔绔?challenge 閹存劕濮涢崥搴″晙閹绘劒姘?| 鏉╂柨娲?`SLG-003` | P0 |
| A-AUTH-008 | challenge 鏉╁洦婀?| 娣囶喗鏁?expires_at 娑撻缚绻冮崢?| 閹绘劒姘︾粵鏃€顢?| 鏉╂柨娲?`SLG-003` | P1 |
| A-AUTH-009 | 闂堢偞纭?answers | answers 闂堢偞鏆熺紒鍕灗鐡掑懓绻冮弫浼村櫤 | 鏉╂柨娲?`SLG-001` | P0 |

### 7.4 SQLite 娑撳骸顓哥拋?
| 缂傛牕褰?| 閻劋绶?| 妤犲矁鐦夐悙?| 娴兼ê鍘涚痪?|
| --- | --- | --- | --- |
| A-SQL-001 | schema 閸掓繂顫愰崠?| 閸掓稑缂撻幍鈧張澶婄秼閸撳秷銆冮敍姝歝hallenge_state`閵嗕梗session_store`閵嗕梗request_store`閵嗕梗nonce_store`閵嗕梗failure_window`閵嗕梗answer_digest_set`閵嗕梗audit_log` | P0 |
| A-SQL-002 | 娑撳秴鍨卞鐑樻＋鐞?| 娑撳秴鐡ㄩ崷?`protected_story_objects` | P0 |
| A-SQL-003 | 缁涙梹顢嶉幗妯款洣閸愭瑥鍙?| `answer_digest_set` 閸欘亙绻氱€?HMAC 閹芥顩﹂敍灞肩瑝娣囨繂鐡ㄩ弰搴㈡瀮缁涙梹顢?| P0 |
| A-SQL-004 | replay 濞夈劌鍞界€孤ゎ吀 | `replay_registered` 閸愭瑥鍙?`audit_log` | P1 |
| A-SQL-005 | challenge 婢惰精瑙︾€孤ゎ吀 | `challenge_failed` 閸愭瑥鍙?`audit_log` | P1 |
| A-SQL-006 | 缁涙儳鎮曢幒鍫熸綀鐎孤ゎ吀 | 缁涙儳鎮曢幋鎰閸愭瑥鍙?`signature_authorized` 鐎孤ゎ吀 | P0 |
| A-SQL-007 | cleanupExpired 濞撳懐鎮?| 鏉╁洦婀?request/nonce 閸掔娀娅庨敍宀冪箖閺?session/challenge 閺嶅洩顔?| P0 |
| A-SQL-008 | cleanup batch 闂勬劕鍩?| batchSize 婢堆傜艾 1000 閺冩湹绮涢梽鎰煑閸?1000 | P1 |
| A-SQL-009 | 閹镐椒绠欓崠鏍ㄦ殶閹诡喖绨?SecretStore 鐟曚焦鐪?| 閹镐椒绠欓崠?dbPath 閺堫亝鏁為崗?SecretStore 閺冭埖瀚嗙紒?| P0 |
| A-SQL-010 | 閺?schema 鏉╀胶些 | 閺冄嗐€冪紒鎾寸€懛顏勫З鐞涖儵缍堥弬鏉垮灙閿涘奔绗栨稉宥嗕划婢跺秵妫弫鍛皑鐎电钖勭悰?| P1 |

## 閸忣偁鈧胶顑囨稉澶婄湴濞村鐦悽銊ょ伐

濞村鐦€电钖勯敍?
1. `StoryLockRemoteGateway`
2. `DelegatedSignatureSkill`
3. 鏉╂粎鈻肩拠閿嬬湴 Schema

### 8.1 requestSignature

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| G-SIGN-001 | 濮濓絽鐖剁粵鎯ф倳鐠囬攱鐪伴崠鍛邦棅 | `identityId`閵嗕梗keyId`閵嗕梗algorithm`閵嗕梗requestId`閵嗕梗nonce`閵嗕梗expiry` | `capability=requestSignature` | P0 |
| G-SIGN-002 | EIP-712 缁鐎风紒鎾寸€?| 閺堝鏅ョ粵鎯ф倳鐠囬攱鐪?| 閸栧懎鎯?`StoryLockSignatureRequest` | P0 |
| G-SIGN-003 | nonce 韫囧懘銆忔稉?uint256 鐎涙顑佹稉?| `eip712Nonce=abc` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| G-SIGN-004 | 閺€顖涘瘮 ed25519 | `algorithm=ed25519` | 鐠囬攱鐪伴柅姘崇箖 | P0 |
| G-SIGN-005 | 閺€顖涘瘮 secp256k1 | `algorithm=secp256k1` | 鐠囬攱鐪伴柅姘崇箖 | P0 |
| G-SIGN-006 | 閹锋帞绮烽棃鐐寸《缁犳纭?| `algorithm=rsa` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| G-SIGN-007 | 鏉╁洦婀＄拠閿嬬湴閹锋帞绮?| `expiry` 鐏忓繋绨ぐ鎾冲閺冨爼妫?| 閹舵稑鍤?`REQUEST_EXPIRED` | P0 |
| G-SIGN-008 | 閺堫剙婀撮幍褑顢戦崳銊ㄧ熅瀵?| 濞夈劌鍙?`signatureExecutor` | 鐠嬪啰鏁ら幍褑顢戦崳銊ヨ嫙鏉╂柨娲栭懘杈ㄦ櫛缂佹挻鐏?| P0 |
| G-SIGN-009 | transport 鐠侯垰绶?| 閺堫亝鏁為崗銉﹀⒔鐞涘苯娅?| 鐠嬪啰鏁?`transport` | P0 |
| G-SIGN-010 | 闁帒缍婇懘杈ㄦ櫛 | 鏉╂柨娲栨稉顓炴儓 `privateKey/password/answers/secretBytes` | 閸忋劑鍎撮弴鎸庡床娑?`[redacted]` | P0 |

### 8.2 requestPasswordFill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| G-PASS-001 | 濮濓絽鐖剁€靛棛鐖滄繅顐㈠帠鐠囬攱鐪?| `identityId`閵嗕梗credentialRef`閵嗕梗targetOrigin` | `capability=requestPasswordFill` | P0 |
| G-PASS-002 | 姒涙顓绘穱婵堟殌缁涙牜鏆?| 閺堚偓鐏忓繑婀侀弫鍫ｇ翻閸?| `requestedRetention=audit_meta_only` | P0 |
| G-PASS-003 | 姒涙顓荤粵鏍殣閹绘劗銇?| 閺堚偓鐏忓繑婀侀弫鍫ｇ翻閸?| `noRemoteSecretReturn=true` | P0 |
| G-PASS-004 | 缂傚搫銇?credentialRef | 娑撳秳绱堕崙顓熷祦瀵洜鏁?| 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| G-PASS-005 | 缂傚搫銇?targetOrigin | 娑撳秳绱堕惄顔界垼缁旀瑧鍋?| 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P0 |
| G-PASS-006 | 閺堫剙婀撮幍褑顢戦崳銊ㄧ熅瀵?| 濞夈劌鍙?`passwordFillExecutor` | 鐠嬪啰鏁ら幍褑顢戦崳銊ヨ嫙閼磋鲸鏅辨潻鏂挎礀 | P0 |
| G-PASS-007 | 閺勫孩鏋冪€靛棛鐖滈懘杈ㄦ櫛 | 閹笛嗩攽閸ｃ劑鏁婄拠顖濈箲閸?`password` | 鏉╂柨娲栨稉顓濊礋 `[redacted]` | P0 |

### 8.3 DelegatedSignatureSkill

| 缂傛牕褰?| 閻劋绶?| 鏉堟挸鍙?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| G-DELEGATE-001 | 婵梹澧粵鎯ф倳閹存劕濮?| 閺堝鏅ラ崣鍌涙殶 | 鐠嬪啰鏁?gateway 閻?`requestSignature` | P1 |
| G-DELEGATE-002 | skillId | 閺?| 鏉╂柨娲?`delegated_signature` | P1 |
| G-DELEGATE-003 | 闂堢偞纭?algorithm | `algorithm=rsa` | 閹舵稑鍤弽锟犵崣闁挎瑨顕?| P1 |

## 娑旀縿鈧浇娉曢崠鍛存肠閹存劖绁寸拠?
瑜版挸澧犲楦款唴閺傛澘顤冪粩顖氬煂缁旑垱绁寸拠鏇″壖閺堫剨绱濇笟瀣洤閿?
`tests/integration/signature-flow.test.mjs`

### 9.1 缁涙儳鎮曢幒鍫熸綀缁旑垰鍩岀粩?
| 缂傛牕褰?| 閻劋绶?| 濞翠胶鈻?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| I-SIGN-001 | 鏉╂粎鈻肩粵鎯ф倳閸掔増婀伴崷鐗堝房閺?| `requestSignature` -> 鐎电钖勫鍝勫缁涙牜鏆?-> 娑旀繂顔傞弽?-> 閺堫剙婀撮幒鍫熸綀 -> signatureExecutor | 鏉╂柨娲栫粵鎯ф倳缂佹挻鐏夐敍灞炬櫛閹扮喎鐡у▓浣冨姎閺?| P0 |
| I-SIGN-002 | 閹哄牊娼堟径杈Е娴肩姵鎸?| 闁挎瑨顕ょ粵鏃€顢?| 缁楊兛绗佺仦鍌涙暪閸掓壆绮ㄩ弸鍕闁挎瑨顕ら敍灞肩瑝濞夊嫰婀剁粵鏃€顢嶇紒鍡氬Ν | P0 |
| I-SIGN-003 | 鐎孤ゎ吀閽€钘夌氨 | 缁涙儳鎮曢幋鎰 | `audit_log` 閺堝顒烽崥宥嗗房閺夊啳顔囪ぐ?| P0 |
| I-SIGN-004 | replay 闂冨弶濮?| 闁插秴顦?requestId/nonce | 鏉╂柨娲?`SLG-013` | P0 |

### 9.2 鐎靛棛鐖滄繅顐㈠帠缁旑垰鍩岀粩?
| 缂傛牕褰?| 閻劋绶?| 濞翠胶鈻?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| I-PASS-001 | 鏉╂粎鈻肩€靛棛鐖滄繅顐㈠帠閸掔増婀伴崷鐗堝房閺?| `requestPasswordFill` -> 鐎电钖勫鍝勫缁涙牜鏆?-> 娑旀繂顔傞弽?-> 閺堫剙婀撮幒鍫熸綀 -> passwordFillExecutor | 鏉╂柨娲栨繅顐㈠帠閹存劕濮涢崗鍐т繆閹?| P0 |
| I-PASS-002 | 娑撳秷绻戦崶鐐存閺傚洤鐦戦惍?| 閹笛嗩攽閸ｃ劏绻戦崶?password 鐎涙顔?| 缁楊兛绗佺仦鍌濆姎閺佸繋璐?`[redacted]` | P0 |
| I-PASS-003 | 闁挎瑨顕ょ粵鏃€顢嶉幏鎺旂卜 | 闁挎瑨顕ょ粵鏃€顢?| 鏉╂柨娲栭幒鍫熸綀婢惰精瑙?| P0 |

## 閸椾降鈧礁鐣ㄩ崗銊ょ瑩妞よ绁寸拠?
| 缂傛牕褰?| 閺€璇插毊閸︾儤娅?| 鏉堟挸鍙?| 妫板嫭婀￠梼鎻掑敖缂佹挻鐏?| 娴兼ê鍘涚痪?|
| --- | --- | --- | --- | --- |
| SEC-001 | 缁岃櫣鐡熷鍫㈢搏鏉?| `answers=[]` | 閹哄牊娼堟径杈Е | P0 |
| SEC-002 | 闂呭繑婧€缁涙梹顢嶉悮婊勭ゴ | 闂呭繑婧€鐎涙顑佹稉?| 婢惰精瑙︾拋鈩冩殶婢х偛濮?| P0 |
| SEC-003 | 鐡掑懘鏆辩粵鏃€顢?| 閸楁洜鐡熷鍫ｇТ鏉?512 鐎涙顑?| 鏉╂柨娲?`SLG-001` | P0 |
| SEC-004 | 鐡掑懎顦跨粵鏃€顢?| 鐡掑懓绻?10 閺夛紕鐡熷?| 鏉╂柨娲?`SLG-001` | P0 |
| SEC-005 | SQL 濞夈劌鍙嗙€涙顑佹稉?| `'; DROP TABLE audit_log; --` | 娑撳秶鐗崸?SQLite 鐞?| P0 |
| SEC-006 | requestId 闁插秵鏂?| 閻╃鎮?requestId閿涘奔绗夐崥?payload | 鏉╂柨娲?`SLG-013` | P0 |
| SEC-007 | nonce 闁插秵鏂?| 娑撳秴鎮?requestId閿涘瞼娴夐崥?nonce | 鏉╂柨娲?`SLG-013` | P0 |
| SEC-008 | 鏉╁洦婀＄拠閿嬬湴 | expiry 鏉╁洦婀?| 鏉╂柨娲?`SLG-011` 閹?`REQUEST_EXPIRED` | P0 |
| SEC-009 | 鐠?identity 閹绘劒姘?| identityB 閻?identityA challenge | 閹哄牊娼堟径杈Е | P0 |
| SEC-010 | 鏉╃偟鐢绘径杈Е閺嗘潙濮忕亸婵婄槸 | 閸?identity 闁?3 濞嗏€蹭簰娑?| 闁夸礁鐣鹃獮鎯扮箲閸?retryAfter | P0 |
| SEC-011 | 鏉╂粎鈻兼潻鏂挎礀閺佸繑鍔呯€涙顔?| `privateKey/password/answers/mnemonic` | 閸忋劑鍎撮懘杈ㄦ櫛 | P0 |
| SEC-012 | 閻㈢喍楠囬幐浣风畽鎼存挻妫?SecretStore | dbPath 閹稿洤鎮滈弬鍥︽娑撴柧绗夊▔銊ュ弳 SecretStore | 閹锋帞绮烽崚娑樼紦 Host | P0 |

## 閸椾椒绔撮妴涓糲hema 娑撳骸顨栫痪锔界ゴ鐠?
瑜版挸澧?Schema 濞撳懎宕熼敍?
| 閸?| Schema |
| --- | --- |
| 閺堫剙婀寸拋鍧楁６閹哄牊娼堥崠?| `access-response.schema.json`閵嗕梗grid-verification-input.schema.json`閵嗕梗local-authorization-input.schema.json`閵嗕梗object-strength-policy-input.schema.json`閵嗕梗selftest-report.schema.json` |
| 閺堫剙婀撮弫鍛皑婢跺嫮鎮婇崠?| `story-draft-input.schema.json`閵嗕梗story-refine-input.schema.json` |
| 鏉╂粎鈻肩純鎴濆彠閸?| `delegated-sign-input.schema.json`閵嗕梗remote-gateway-request.schema.json`閵嗕梗remote-gateway-response.schema.json` |
| 閸忕厧顔愬鏃傘仛閸?| `password-fill-input.schema.json`閵嗕梗password-fill-output.schema.json`閵嗕梗story-draft-input.schema.json`閵嗕梗strength-review-output.schema.json` |

婵傛垹瀹冲ù瀣槸妞ょ櫢绱?
| 缂傛牕褰?| 閻劋绶?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- |
| C-SCHEMA-001 | 閹碘偓閺?JSON Schema 閸欘垵顫﹂弽鍦窗瑜版洖顨栫痪锕佸壖閺堫剙濮炴潪?| 閺冪姾顕㈠▔鏇㈡晩鐠?| P1 |
| C-SCHEMA-002 | 閺堝鏅?object strength 鏉堟挸鍙?| 闁俺绻冮弽锟犵崣 | P1 |
| C-SCHEMA-003 | 閺堝鏅?grid verification 鏉堟挸鍙?| 闁俺绻冮弽锟犵崣 | P1 |
| C-SCHEMA-004 | 閺堝鏅?local authorization 鏉堟挸鍙?| 闁俺绻冮弽锟犵崣 | P1 |
| C-SCHEMA-005 | 閺堝鏅?remote gateway request | 娴犲懎鍘戠拋?`requestSignature`閵嗕梗requestPasswordFill` | P0 |
| C-SCHEMA-006 | access response 闁挎瑨顕ら惍浣圭壐瀵?| `SLG-xxx` 閺嶇厧绱￠柅姘崇箖 | P1 |
| C-DOC-001 | 闁挎瑨顕ら惍浣规瀮濡楋絼绔撮懛?| 娑?`errors.js` 娑?`ERROR_DEFS` 娑撯偓閼?| P1 |
| C-DOC-002 | 娑撳秴鍤悳鐗堟＋娑撹崵鍤庨幒銉ュ經 | 閺傚洦銆傛稉宥呯繁閹跺﹥妫幒銉ュ經閸愭瑦鍨氳ぐ鎾冲娑撹崵鍤?| P0 |
| C-DOC-003 | Node 閻楀牊婀版稉鈧懛?| 閺傚洦銆傞崪?package.json 閸у洩顩﹀Ч?Node.js 22+ | P0 |

## 閸椾椒绨╅妴浣稿悑鐎硅绱ㄧ粈鍝勫瘶濞村鐦?
濞村鐦€电钖勯敍?
`src/engine`

| 缂傛牕褰?| 閻劋绶?| 妫板嫭婀＄紒鎾寸亯 | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- |
| E-COMPAT-001 | `npm run selftest` | 闁俺绻?| P1 |
| E-COMPAT-002 | `LocalPasswordFillSkill` | 鏉╂柨娲?`mode=local_password_fill` | P1 |
| E-COMPAT-003 | `SignatureAuthorizationSkill` | 鏉╂柨娲?`mode=signature_authorization` | P1 |
| E-COMPAT-004 | 妞よ泛鐪扮€电厧鍤Λ鈧弻?| 娑撳秵濡搁弮?challenge sign 娴ｆ粈璐熸稉鑽ゅ殠鐎电厧鍤?| P1 |
| E-COMPAT-005 | WASM 閼奉亝绁?| 婵″倸鐡ㄩ崷銊︾€杞伴獓閻椻晪绱濇潻鎰攽 `npm run selftest:wasm` | P2 |

## 閸椾椒绗侀妴浣圭ゴ鐠囨洘鏆熼幑顔款啎鐠?
### 13.1 閸╄櫣顢呴煬顐″敜

```js
export const testIdentity = {
  identityId: 'id-test-001',
};
```

### 13.2 缁涙梹顢嶉惂鏄忣唶

缁楊兛绨╃仦鍌涚ゴ鐠囨洖绻€妞よ鍘涢惂鏄忣唶缁涙梹顢嶉幗妯款洣閿?
```js
host.enrollAnswers('id-test-001', [
  'correct grid answer',
  'backup answer',
]);
```

濞村鐦稉顓熷絹娴溿倗鐡熷鍫窗

```js
[
  { cellId: 'cell-1', answer: 'correct grid answer' }
]
```

### 13.3 鐠囬攱鐪?envelope

```js
export function makeEnvelope(prefix = 'req') {
  const suffix = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return {
    requestId: `${prefix}-${suffix}`,
    nonce: `nonce-${suffix}`,
    expiry: Date.now() + 60_000,
  };
}
```

### 13.4 娑撳瓨妞?SQLite

```js
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { randomUUID } from 'node:crypto';

export function tempDbPath() {
  return join(tmpdir(), `storylock_test_${randomUUID().replaceAll('-', '')}.db`);
}
```

## 閸椾礁娲撻妴浣哥紦鐠侇喗绁寸拠鏇犳窗瑜?
瑜版挸澧犻崣顖氬帥娣囨繄鏆€閸氬嫬瀵?`scripts/selftest.mjs`閵嗗倸鎮楃紒顓″閹碘晛鐫嶅锝呯础濞村鐦惄顔肩秿閿涘苯缂撶拋顔碱洤娑撳绱?
```text
skill/
  tests/
    integration/
      signature-flow.test.mjs
      password-fill-flow.test.mjs
    security/
      replay.test.mjs
      redaction.test.mjs
      lockout.test.mjs
    contract/
      schemas.test.mjs
      docs-code-consistency.test.mjs
  src/
    storylock-local-story-processing-skill/
      scripts/selftest.mjs
    storylock-local-story-access-skill/
      scripts/selftest.mjs
    storylock-remote-gateway-skill/
      scripts/selftest.mjs
    storylock-skill-engine/
      scripts/selftest.mjs
```

## 閸椾椒绨查妴浣瑰⒔鐞涘矁顓搁崚?
| 闂冭埖顔?| 閸愬懎顔?| 娴溠呭⒖ | 娴兼ê鍘涚痪?|
| --- | --- | --- | --- |
| T0 | 鐠烘垿鈧氨骞囬張澶婃磽娑?selftest | 瑜版挸澧犻崘鎺斿劔閸╄櫣鍤?| P0 |
| T1 | 鐞涖儳顏崚鎵伂缁涙儳鎮曢柧鎹愮熅濞村鐦?| `signature-flow.test.mjs` | P0 |
| T2 | 鐞涖儳顏崚鎵伂鐎靛棛鐖滄繅顐㈠帠闁炬崘鐭惧ù瀣槸 | `password-fill-flow.test.mjs` | P0 |
| T3 | 鐞涖儲妲楃€瑰缍夌粩娆愭付鐏忓繗鍤滃Λ鈧?| 瀹稿弶鏌婃晶?`src/ui/scripts/selftest-site.mjs` 楠炶埖甯撮崗銉︾壌 `npm run selftest` | P0 |
| T4 | 鐞?APK 娑撳娴囨稉搴″帗閺佺増宓侀懛顏咁梾 | 瀹稿弶澧跨仦?`selftest:web-api-android` 鐟曞棛娲婃稉瀣祰閵嗕胶澧楅張顑锯偓浣搞亣鐏忓繈鈧恭hecksum閵嗕礁瀵樼猾璇茬€烽妴浣稿絺鐢啴鈧岸浜?| P0 |
| T5 | 鐞涖儱鐣ㄩ崗銊ょ瑩妞よ绁寸拠?| replay閵嗕勾ockout閵嗕购edaction 濞村鐦?| P0 |
| T6 | 鐞?Schema 婵傛垹瀹冲ù瀣槸 | 瀹稿弶鏌婃晶?`npm run test:contract`閿涘苯鎮楃紒顓炲讲缂佈呯敾閹恒儱鍙?Ajv | P1 |
| T7 | 鐞涖儲鏋冨?娴狅絿鐖滄稉鈧懛瀛樷偓褎绁寸拠?| 闂堟瑦鈧焦澹傞幓蹇氬壖閺?| P1 |
| T8 | 鐞涖儴顩惄鏍芳缂佺喕顓?| 鐟曞棛娲婇悳鍥ㄥГ閸?| P2 |

## 閸椾礁鍙氶妴渚€鈧俺绻冮弽鍥у櫙

### 16.1 P0 闁俺绻冮弽鍥у櫙

1. 閸ユ稐閲滈崠?`npm run selftest` 閸忋劑鍎撮柅姘崇箖閵?2. 缁楊兛绨╃仦鍌炴晩鐠囶垳鐡熷鍫滅瑝閼宠姤宸块弶鍐︹偓?3. requestId 閹?nonce 閸愯尙鐛婃潻鏂挎礀 `SLG-013`閵?4. 鏉╁洦婀＄拠閿嬬湴鏉╂柨娲?`SLG-011` 閹存牗濮忛崙?`REQUEST_EXPIRED`閵?5. 鏉╂粎鈻兼潻鏂挎礀娑擃厽鏅遍幇鐔风摟濞堜絻顫﹂柅鎺戠秺閼磋鲸鏅遍妴?6. SQLite 娑擃厺绗夋穱婵嗙摠閺勫孩鏋冪粵鏃€顢嶉妴?7. 缁涙儳鎮曢幒鍫熸綀鐎孤ゎ吀閸愭瑥鍙?`audit_log`閵?8. 閺傚洦銆傞崪?Schema 娑撳秵濡搁弮褎甯撮崣锝勭稊娑撳搫缍嬮崜宥勫瘜缁捐￥鈧?
### 16.2 P1 闁俺绻冮弽鍥у櫙

1. JSON Schema 閸忋劑鍎撮崣顖氬鏉炲鈧?2. 娑撴槒顕Ч鍌氭嫲閸濆秴绨查弽铚傜伐閼充粙鈧俺绻?Schema 閺嶏繝鐛欓妴?3. 闁挎瑨顕ら惍浣姐€冩稉?`errors.js` 娑撯偓閼锋番鈧?4. 閸忕厧顔愬鏃傘仛閸栧懎鐔€绾偓閼宠棄濮忛崣顖濈箥鐞涘被鈧?
### 16.3 P2 闁俺绻冮弽鍥у櫙

1. 瑜般垺鍨氱憰鍡欐磰閻滃洦濮ら崨濞库偓?2. 閸旂姴鍙?CI 閼奉亜濮╅幍褑顢戦妴?3. 婢х偛濮為崢瀣濞村鐦幋鏍毐閺冨爼妫挎潻鎰攽濞村鐦妴?
## 閸椾椒绔烽妴涓咺 瀵ら缚顔?
瑜版挸澧犳禒鎾崇氨濞屸剝婀佺紒鐔剁閺?`package.json` 濞村鐦崗銉ュ經閺冭绱濋崣顖氬帥娴ｈ法鏁?PowerShell 閼存碍婀伴敍?
```powershell
$ErrorActionPreference = "Stop"

Push-Location src/skills/local-story-processing
npm run selftest
Pop-Location

Push-Location src/skills/local-story-access
npm run selftest
Pop-Location

Push-Location src/skills/remote-gateway
npm run selftest
Pop-Location

Push-Location src/engine
npm run selftest
Pop-Location
```

閸氬海鐢婚崣顖氼杻閸旂姵鐗撮惄顔肩秿閼存碍婀伴敍?
```json
{
  "scripts": {
    "test:self": "powershell -ExecutionPolicy Bypass -File scripts/runtime/test_self.ps1",
    "test:integration": "node --test tests/integration/*.test.mjs",
    "test:security": "node --test tests/security/*.test.mjs",
    "test:contract": "node --test tests/contract/*.test.mjs"
  }
}
```

## 閸椾礁鍙撻妴渚€顥撻梽鈺€绗屾惔鏂款嚠

| 妞嬪酣娅?| 瑜板崬鎼?| 鎼存柨顕?|
| --- | --- | --- |
| 濞村鐦弬瑙勵攳濞岃法鏁ら弮褎甯撮崣?| 濞村鐦径杈╂埂 | 閺堫剚鏌熷鍫濆嚒缁夊娅庨弫鍛皑鐠囪鍟撻崪?challenge sign 娑撹崵鍤庡ù瀣槸 |
| `node:sqlite` 鏉╂劘顢戦弮璺烘▕瀵?| 閼奉亝绁撮弮鐘崇《鏉╂劘顢?| 閸ュ搫鐣?Node.js 22+ |
| Mock 娑撳海婀＄€?SQLite 鐞涘奔璐熸稉宥勭閼?| 闂嗗棙鍨氱紓娲濠曞繑绁?| P0 闂嗗棙鍨氬ù瀣槸韫囧懘銆忔担璺ㄦ暏閻喎鐤勬稉瀛樻 SQLite |
| 缁旑垰鍩岀粩顖炴懠鐠侯垰鐨婚張顏勭暚閺佺鍤滈崝銊ュ | 閸欏倽绂屽鏃傘仛妞嬪酣娅?| 娴兼ê鍘涚悰?`signature-flow` 閸?`password-fill-flow` |
| SecretStore 閻㈢喍楠囬柅鍌炲帳閺堫亜鐣幋?| 閻㈢喍楠囩€瑰鍙忔搴ㄦ珦 | 閹镐椒绠欓崠鏍ㄧゴ鐠囨洖绻€妞ゆ槒顩惄鏍ㄦ￥ SecretStore 閹锋帞绮烽柅鏄忕帆 |

## 閸椾椒绡€閵嗕線妾ぐ鏇窗瑜版挸澧犻柨娆掝嚖閻焦绁寸拠鏇炲經瀵?
| 闁挎瑨顕ら惍?| 缁鐎?| 濞村鐦柌宥囧仯 |
| --- | --- | --- |
| `SLG-001` | `validation_error` | 鐎涙顔岀紓鍝勩亼閵嗕胶琚崹瀣晩鐠囶垬鈧線鏆辨惔锕佺Т闂?|
| `SLG-002` | `replay_rejected` | 娣囨繄鏆€閸樺棗褰堕崗鐓庮啇閿涘奔绗夋担婊€璐熻ぐ鎾冲 replay 閸愯尙鐛婃稉鑽ょ垳 |
| `SLG-003` | `challenge_failed` | 缁涙梹顢嶉柨娆掝嚖閵嗕恭hallenge 閺冪姵鏅?|
| `SLG-004` | `challenge_locked` | 鏉╃偟鐢绘径杈Е闁夸礁鐣?|
| `SLG-005` | `session_invalid` | session 娑撳秴鐡ㄩ崷銊ｂ偓浣界箖閺堢喐鍨ㄩ悩鑸碘偓浣规￥閺?|
| `SLG-006` | `budget_exhausted` | 妫板嫮鐣婚懓妤€鏁?|
| `SLG-007` | `object_not_found` | 鐎电钖勬稉宥呯摠閸?|
| `SLG-008` | `redaction_required` | 缂佹挻鐏夎箛鍛淬€忛懘杈ㄦ櫛 |
| `SLG-009` | `scope_insufficient` | scope 娑撳秷鍐?|
| `SLG-010` | `secret_unavailable` | SecretStore 閹存牜鐡熷鍫熸喅鐟曚椒绗夐崣顖滄暏 |
| `SLG-011` | `request_expired` | 鐠囬攱鐪版潻鍥ㄦ埂 |
| `SLG-012` | `internal_error` | 閺堫亜鍨庣猾璇插敶闁劑鏁婄拠?|
| `SLG-013` | `replay_detected` | requestId 閹?nonce 閸愯尙鐛?|

## 娴滃苯宕勯妴渚€妾ぐ鏇窗娑撳秵绁寸拠鏇氳礋瑜版挸澧犻懗钘夊閻ㄥ嫬鍞寸€?
娴犮儰绗呴崘鍛啇娑撳秶鎾奸崗銉ョ秼閸?v1.0 娑撹崵鍤庢灞炬暪閿?
Deprecated historical interfaces only; no longer in current mainline:
1. 娑撳秵绁寸拠鏇熸櫊娴滃顕挒陇顕伴崣鏍モ偓?2. 娑撳秵绁寸拠鏇熸櫊娴滃顕挒鈥冲晸閸ョ偑鈧?3. 娑撳秵绁寸拠鏇炲嚒鎼寸喎绱旈弮褎甯撮崣?`requestChallengeSign`閵?4. 鐎瑰本鏆ｉ柧鍙ョ瑐妤犲矁鐦夐妴?5. EIP-1271/ERC-4337 閻㈢喍楠囩痪褔娉﹂幋鎰┾偓?6. 婢舵岸鎽奸妴浣割樋闁藉崬瀵橀妴浣割樋鐠愶箑褰挎惔鏃傛暏缂傛牗甯撻妴?
鏉╂瑤绨洪崘鍛啇閸欘亣鍏樻担婊€璐熼崥搴ｇ敾鎼存梻鏁ら幒銏㈠偍閹存牗澧跨仦鏇熺ゴ鐠囨洟銆嶉妴?

