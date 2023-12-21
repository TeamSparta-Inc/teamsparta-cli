// preinstall script에 포함되는 파일은 dependencies가 설치되기 전에 실행된다.
// 그러므로, 존재하지 않을 경우 단지 에러를 무시하기 위해 아래와 같은 방식으로 처리한다.

function getBinary() {
  try {
    // dependency가 존재하기 전일 경우, 에러가 발생한다.
    const getBinary = require("./getBinary");

    return getBinary();
  } catch {}
}

const binary = getBinary();
binary && binary.uninstall();
