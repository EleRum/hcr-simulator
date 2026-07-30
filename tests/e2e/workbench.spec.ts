import { expect, test } from '@playwright/test';

test.describe('HCR Simulator workbench', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(
      page.getByRole('heading', { name: 'HCR Simulator' }),
    ).toBeVisible();
    await expect(page.getByTestId('blockly-editor')).toBeVisible();
    await expect(page.getByTestId('simulator-canvas')).toBeVisible();
    await expect(page.getByTestId('simulator-canvas')).toHaveAttribute(
      'data-render-state',
      'ready',
    );
    await expect(page.getByTestId('current-voxel-count')).toHaveText(
      '241',
    );
  });

  test('runs the starter program to a reproducible scored result', async ({
    page,
  }) => {
    const pageErrors: Error[] = [];
    page.on('pageerror', (error) => pageErrors.push(error));
    const blockCountBefore = await page
      .locator('.blocklyBlockCanvas .blocklyDraggable')
      .count();
    await page.getByTestId('run-button').click();

    await expect(page.getByTestId('simulation-status')).toHaveText(
      '运行中',
    );
    await expect(page.getByTestId('simulator-canvas')).toHaveAttribute(
      'data-render-state',
      'ready',
    );
    await expect(
      page
        .getByTestId('simulator-canvas')
        .locator('canvas'),
    ).toHaveCSS('background-color', 'rgb(10, 20, 29)');
    await expect(page.getByText('程序执行期间编辑已锁定')).toBeVisible();
    await expect(page.locator('.blocklyHighlighted')).toHaveCount(1);
    await expect(page.getByTestId('simulation-status')).toHaveText(
      '已完成',
      { timeout: 15_000 },
    );
    await expect(page.getByTestId('current-voxel-count')).toHaveText(
      '217',
    );
    await expect(page.getByTestId('executed-command-count')).toHaveText(
      '10',
    );
    await expect(page.getByTestId('final-score')).toBeVisible();
    await expect(page.getByTestId('simulator-canvas')).toHaveAttribute(
      'data-render-state',
      'ready',
    );

    const completion = Number(
      await page.getByTestId('completion-score').textContent(),
    );
    expect(completion).toBeGreaterThanOrEqual(80);

    await page.getByTestId('reset-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText('待机');
    await expect(page.getByTestId('current-voxel-count')).toHaveText(
      '241',
    );
    await expect(page.getByTestId('final-score')).toHaveCount(0);
    await expect(
      page.locator('.blocklyBlockCanvas .blocklyDraggable'),
    ).toHaveCount(blockCountBefore);
    expect(pageErrors).toEqual([]);
  });

  test('pauses, advances one command, resumes and records events', async ({
    page,
  }) => {
    await page.getByTestId('run-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText(
      '运行中',
    );
    await page.getByTestId('pause-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText(
      '已暂停',
    );

    const countBeforeStep = Number(
      await page.getByTestId('executed-command-count').textContent(),
    );
    await page.waitForTimeout(350);
    await expect(page.getByTestId('executed-command-count')).toHaveText(
      String(countBeforeStep),
    );

    await page.getByTestId('step-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText(
      '已暂停',
      { timeout: 5_000 },
    );
    const countAfterStep = Number(
      await page.getByTestId('executed-command-count').textContent(),
    );
    expect(countAfterStep).toBe(countBeforeStep + 1);

    await page.getByTestId('resume-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText(
      '已完成',
      { timeout: 15_000 },
    );

    await page.getByTestId('log-toggle').click();
    await expect(page.getByTestId('event-log')).toContainText(
      '评分完成',
    );
    await expect(page.getByTestId('event-log')).toContainText(
      '程序已暂停',
    );
    await expect(page.getByTestId('event-log')).toContainText('剪除');
  });

  test('stops without a formal score and preserves reset behavior', async ({
    page,
  }) => {
    await page.getByTestId('run-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText(
      '运行中',
    );
    await page.getByTestId('stop-button').click();

    await expect(page.getByTestId('simulation-status')).toHaveText(
      '已停止',
    );
    await expect(page.getByText(/不生成正式成绩/)).toBeVisible();
    await expect(page.getByTestId('final-score')).toHaveCount(0);

    await page.getByTestId('reset-button').click();
    await expect(page.getByTestId('simulation-status')).toHaveText('待机');
    await expect(page.getByTestId('current-voxel-count')).toHaveText(
      '241',
    );
  });

  test('toggles the target preview', async ({ page }) => {
    const toggle = page.getByRole('button', { name: /目标发型预览/ });
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-pressed', 'false');
  });

  test('shows a local recovery state when WebGL context is lost', async ({
    page,
  }) => {
    const canLoseContext = await page
      .getByTestId('simulator-canvas')
      .locator('canvas')
      .evaluate((canvas: HTMLCanvasElement) => {
        const gl =
          canvas.getContext('webgl2') ?? canvas.getContext('webgl');
        const extension = gl?.getExtension('WEBGL_lose_context');
        extension?.loseContext();
        return Boolean(extension);
      });

    test.skip(!canLoseContext, 'WEBGL_lose_context is unavailable');
    await expect(page.getByRole('alert')).toContainText(
      '3D 渲染已中断',
    );
    await page
      .getByRole('button', { name: '重新初始化 3D' })
      .click();
    await expect(page.getByTestId('simulator-canvas')).toHaveAttribute(
      'data-render-state',
      'ready',
    );
  });

  test('keeps the primary workspace controls visible at desktop sizes', async ({
    page,
  }) => {
    await expect(page.getByTestId('run-button')).toBeInViewport();
    await expect(page.getByTestId('reset-button')).toBeInViewport();
    await expect(page.getByTestId('blockly-editor')).toBeInViewport();
    await expect(page.locator('.side-panel--right')).toBeInViewport();
    await expectReadableFontSizes(page);

    await page.setViewportSize({ width: 1920, height: 1080 });
    await expect(page.getByTestId('run-button')).toBeInViewport();
    await expect(page.getByTestId('reset-button')).toBeInViewport();
    await expect(page.getByTestId('blockly-editor')).toBeInViewport();
    await expect(page.getByTestId('simulator-canvas')).toBeInViewport();
    await expect(page.locator('.side-panel--right')).toBeInViewport();
    await expectReadableFontSizes(page);
  });
});

async function expectReadableFontSizes(
  page: import('@playwright/test').Page,
): Promise<void> {
  const selectors = [
    '.panel-header span',
    '.panel-header strong',
    '.joint-row strong',
    '.metric-card span',
    '.control-button',
    '.blocklyToolboxCategoryLabel',
  ];

  for (const selector of selectors) {
    const fontSize = await page.locator(selector).first().evaluate(
      (element) =>
        Number.parseFloat(getComputedStyle(element).fontSize),
    );
    expect(fontSize, `${selector} font size`).toBeGreaterThanOrEqual(11);
  }
}
