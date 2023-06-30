# Team Sparta CLI

1. dump: 프로덕션 데이터베이스로부터 로컬로 덤프
   - service(required)
   - collection(optional 명시하지 않을 경우 기본 설정대로 덤프)
   - port(optional)
2. watch: 서비스 배포 관측. codepipeline 프로세스가 완료될 경우 슬랙으로 멘션 알림
   - service(required)
   - notify(optional 명시하지 않을 경우 자신에게만 알림)
3. unwatch: watch daemon process를 종료시킨다

   - service(required)

4. image-resize(개발 예정): 프론트 커밋 시 자동으로 이미지 파일 사이즈 감지 및 변경
   - target(required)
5. ssh(개발 예정): ssh 커맨드라인 축약
   - service(required)
6. onboarding(개발 예정): 온보딩 프로세스 출력 및 체크
   - done(optional)
7. check(개발 예정): 필수 자원(pem key, AWS config, git config 등)위치 확인
   - update(optional)

# ⚡️ Rust-Powerd ⚡️
