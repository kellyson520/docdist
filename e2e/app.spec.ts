import { test, expect } from '@playwright/test';

test.describe('追光 Lite', () => {
  test('应用加载成功', async ({ page }) => {
    await page.goto('/');
    
    // 等待应用加载
    await expect(page).toHaveTitle(/追光/);
  });

  test('侧边栏导航显示正确', async ({ page }) => {
    await page.goto('/');
    
    // 检查导航项
    await expect(page.getByText('存档管理')).toBeVisible();
    await expect(page.getByText('时间轴')).toBeVisible();
    await expect(page.getByText('版本对比')).toBeVisible();
    await expect(page.getByText('迭代图谱')).toBeVisible();
  });

  test('新建存档按钮存在', async ({ page }) => {
    await page.goto('/');
    
    // 检查新建存档按钮
    await expect(page.getByText('新建存档')).toBeVisible();
  });
});
