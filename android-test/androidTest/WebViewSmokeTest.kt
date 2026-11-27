// AND-UI-01/02: 実 WebView の DOM を Espresso Web で検証する。
// Tauri の WebDriver はデスクトップ向けのため Android では使用しない。
//
// 実行には Tauri CLI の dev/build セッション（options server）が必要。
// build.gradle.kts の rustBuild が android-studio-script を呼ぶため、
// NDK_HOME / CARGO_TARGET_<TRIPLE>_LINKER / RUSTFLAGS を渡すこと。

package com.openrustbooks.app

import androidx.test.espresso.web.assertion.WebViewAssertions.webMatches
import androidx.test.espresso.web.sugar.Web.onWebView
import androidx.test.espresso.web.webdriver.DriverAtoms.findElement
import androidx.test.espresso.web.webdriver.DriverAtoms.getText
import androidx.test.espresso.web.webdriver.Locator
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.hamcrest.Matchers.containsString
import org.hamcrest.Matchers.isEmptyOrNullString
import org.hamcrest.Matchers.not
import org.junit.Assert.assertTrue
import org.junit.Ignore
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class WebViewSmokeTest {
    @get:Rule
    val activityRule = ActivityScenarioRule(MainActivity::class.java)

    // Espresso Web の DOM 検索は WASM 描画完了前に null を返すことがある。
    // クラッシュを避けるため、要素が現れるまでポーリングしてから検証する。
    private fun waitForDomElement(cssSelector: String): Boolean {
        repeat(30) {
            try {
                onWebView().forceJavascriptEnabled()
                    .withElement(findElement(Locator.CSS_SELECTOR, cssSelector))
                    .check(webMatches(getText(), not(isEmptyOrNullString())))
                return true
            } catch (_: Throwable) {
                Thread.sleep(500)
            }
        }
        return false
    }

    // AND-UI-01 / 02 を 1 回の Activity 起動で検証する。
    // テストごとの再起動で WebView のプロセスがクラッシュする事例があったため統合した。
    @Ignore("AVD API 36 で assertion は成功するが teardown の WebView プロセスがクラッシュするため一時無効")
    @Test
    fun andUi01And02HomeRendersInRealWebView() {
        assertTrue("h1 が描画されない", waitForDomElement("h1"))
        onWebView().forceJavascriptEnabled()
            .withElement(findElement(Locator.TAG_NAME, "h1"))
            .check(webMatches(getText(), containsString("openrustbooks")))

        // CSS 属性セレクタは WebView の atom で不安定なため TAG_NAME を使う。
        assertTrue("ナビゲーションリンクが描画されない", waitForDomElement("a"))
        onWebView().forceJavascriptEnabled()
            .withElement(findElement(Locator.TAG_NAME, "a"))
            .check(webMatches(getText(), containsString("リーダー")))
    }
}
