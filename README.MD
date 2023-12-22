# Team Sparta CLI

## config파일이 없는 경우 동료 개발자들에게 문의해주세요

1. dump: 프로덕션 데이터베이스로부터 로컬로 덤프

   - service, s: string enum
   - collection, c: string (optional 명시하지 않을 경우 config.json에서 exclude로 지정된 콜렉션들을 제외하고 모두 덤프)
   - port, p: number (optional)

2. resize: 이미지 파일 해상도 변경

   - input-dir, i: path string
   - output-dir, o: path string
   - file-name, f: string(optional 명시하지 않을 경우 input-dir 내 모든 jpeg와 png파일을 resize \*.{jpeg|png})
   - width, w: number(u32)
   - height, h: number(u32)

3. compress: png파일 무손실/손실 압축

   (**손실 압축시 파일 크기는 매우 작아지지만 컬러가 RGB8로 고정됩니다(tiny-png방식. quantize 알고리즘 사용)**)

   - input-dir, i: path string
   - output-dir, o: path string
   - file-name, f: string (optional 명시하지 않을 경우 input-dir 내 모든 png 파일 압축)
   - (무손실 압축시) level, l: number (optional 압축율(1~12). 높을수록 더 압축. 기본값 12)
   - drop-color, d: 손실 압축 명시
     - (손실 압축시) quality, q: 품질 최댓값(optional 1~100)(default:65, 최솟값은 0 고정)
     - (손실 압축시) speed, s: 압축 속도(optional 1~10, 기본 값: 4, 속도를 희생해서 압축률을 높일 수 있습니다)

4. webpify

   - input-dir, i: path string
   - output-dir, o: path string

5. ssh(개발 예정): ssh 커맨드라인 축약
   - service, s: string enum
6. onboarding(개발 예정): 온보딩 프로세스 출력 및 체크
   - done, d: invoke (optional)
7. check(개발 예정): 필수 자원(pem key, AWS config, git config 등)위치 확인
   - update, u: invoke (optional)

# ⚡️ Rust-Powered ⚡️
