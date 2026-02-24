import { config } from "dotenv";

config();

/**
 * メインスクリプト
 */
export const createGithubIssue = async () => {
  const token = process.env.GITHUB_ACCESS_TOKEN;
  const userName = process.env.GITHUB_USER_NAME;
  const repoName = process.env.GITHUB_REPO_NAME;

  const url = `https://api.github.com/repos/${userName}/${repoName}/issues`;

  const data = {
    title: 'test Issue',
    body: 'testです',
    labels: ['bug'],
  };

  // 実行
  fetch(url, {
    method: 'POST',
    headers: {
      'Authorization': `token ${token}`,
      'Content-Type': 'application/json',
      'User-Agent': 'Node.js',
    },
    body: JSON.stringify(data),
  })
    .then((res) => { return res.json(); })
    .then((body) => console.log(body))
    .catch((e) => console.error(e));
};