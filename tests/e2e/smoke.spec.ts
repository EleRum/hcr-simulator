import { expect, test } from '@playwright/test';

test('loads the HCR Simulator shell', async ({ page }) => {
  await page.goto('/');

  await expect(
    page.getByRole('heading', { name: 'HCR Simulator' }),
  ).toBeVisible();
});
