# Team Sparta CLI

1. dump: 프로덕션 데이터베이스로부터 로컬로 덤프
   - service, s: string enum
   - collection, c: string (optional 명시하지 않을 경우 config.json에서 exclude로 지정된 콜렉션들을 제외하고 모두 덤프)
   - port, p: number (optional)
2. watch: 서비스 배포 관측. codepipeline 프로세스가 완료될 경우 슬랙으로 멘션 알림
   - service, s: string enum
   - notify, n: multiple string(optional 명시하지 않을 경우 자신에게만 알림)
3. unwatch: watch daemon process를 종료시킨다
   - service, s: string enum
4. resize: 이미지 파일 해상도 변경

   - input-dir, i: path string
   - output-dir, o: path string
   - file-name, f: string(optional 명시하지 않을 경우 input-dir 내 모든 jpeg와 png파일을 압축 \*.{jpeg|png})
   - width, w: number(u32)
   - height, h: number(u32)

5. compress: png파일 무손실/손실 압축
   **손실 압축시 파일 크기는 매우 작아지지만 컬러가 RGB8로 고정됩니다(tiny-png방식. quantize 알고리즘 사용)**

   - input-dir, i: path string
   - output-dir, o: path string
   - file-name, f: string (optional 명시하지 않을 경우 input-dir 내 모든 png 파일 압축)
   - (무손실 압축시) level, l: number (optional 압축율(1~12). 높을수록 더 압축. 기본값 12)
   - --drop-color: 손실 압축 명시 (축약형 명령어 없음)
     - (손실 압축시) quality, q: 품질 최댓값(optional 1~100)(default:65, 최솟값은 0 고정)
     - (손실 압축시) speed, s: 압축 속도(optional 1~10, 기본 값: 4, 속도를 희생해서 압축률을 높일 수 있습니다)

6. ssh(개발 예정): ssh 커맨드라인 축약
   - service, s: string enum
7. onboarding(개발 예정): 온보딩 프로세스 출력 및 체크
   - done, d: invoke (optional)
8. check(개발 예정): 필수 자원(pem key, AWS config, git config 등)위치 확인
   - update, u: invoke (optional)

# ⚡️ Rust-Powered ⚡️
