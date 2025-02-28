## 不安全的 Rust

到目前為止，我們討論的所有程式碼都在編譯期強制加上 Rust 記憶體安全保證。然而，Rust 內部其實隱藏了第二種語言，並不強制加上這些記憶體安全保證：這語言叫做**不安全（unsafe）的 Rust**，可和常規 Rust 一樣正常執行，同時賦予我們極強的能力。

不安全的 Rust 之所以存在，是由於靜態分析本質上過於保守。當編譯器嘗試確認程式碼是否遵守這些安全保證時，比起接受一些非法的程式，更寧願拒絕部分合法程式。儘管有些程式碼**看起來**正確，但 Rust 無法獲取足夠資訊保證的話，它就是會擋下來。在這些案例中，你可以寫不安全程式碼並告訴編譯器：「相信我，我知道我在幹麻。」從反面來看這也有缺點，你必須自行承擔風險：若誤用不安全程式碼，可能會造成記憶體不安全，例如發生對空指標（null pointer）解引用。

Rust 擁有另一個不安全的自我的另一理由是電腦硬體本質上就不安全。如果 Rust 不允許這些不安全操作，就無法完成特定任務。Rust 必須允許你做這些底層系統程式設計，例如直接與作業系統互動，甚至撰寫自己的作業系統。系統程式設計是這個語言的目標之一，一起探索我們可以用不安全的 Rust 做什麼和如何使用吧。

### 不安全的超能力

欲切換成不安全的 Rust，可使用 `unsafe` 關鍵字開啟一個新程式碼區塊，並封裝這些不安全程式碼。在不安全的 Rust，你可使用在安全的 Rust 之下無法使用的五種功能，我們稱之為**不安全的超能力**。這些超能力包含：

* 對裸指標（raw pointer）解引用
* 呼叫不安全函式或方法
* 存取或修改可變的靜態變數（static variable）
* 實作不安全特徵（trait）
* 存取聯合體（union）的欄位

需要謹記在心的是，`unsafe` 並不會關閉借用檢查器（borrow checker）或是停用其他 Rust 的安全檢查：在不安全程式碼中操作一個引用仍然會經過檢查。`unsafe` 關鍵字只提供上述不經由編譯器檢查記憶體安全的五項功能，在不安全區塊內你依然保有一定程度的安全性。

此外，`unsafe` 並不意味在此區塊內的程式碼一定有風險或有記憶體安全問題：其目的是作為一個程式設計師，你必須確保在 `unsafe` 區塊內的程式碼透過合法途徑存取記憶體。

錯誤因人類不可靠而發生。不過，將五種不安全操作標記在 `unsafe` 區塊內，讓你得知任何記憶體安全相關的錯誤一定在某個 `unsafe` 內。請將 `unsafe` 區塊保持夠小，當你在調查一個記憶體錯誤時，會慶幸當初有這麼做。

為了盡可能隔離不安全程式碼，最佳作法是將之封裝在安全的抽象並提供安全的 API，本章在後面的探討不安全函式和方法一併討論之。部分的標準函式庫同樣是在審核過的不安全程式碼上提供安全抽象。透過安全抽象封裝不安全程式碼，可防止你或你的使用者使用以 `unsafe` 實作的功能，不會將實際的 `unsafe` 使用洩漏到四散各地，因為安全抽象就是安全的 Rust。

接下來將依序探討這五個不安全的超能力。也會看看一些替不安全程式碼提供安全介面的抽象。

### 對裸指標解引用

在第四章[「迷途引用」][迷途引用]一節，我們提及編譯器確保引用一定是合法的。不安全的 Rust 有兩種新型別叫**裸指標**，和引用非常相似。和引用一樣，裸指標能是不可變或可變，分別寫做 `*const T` 和 `*mut T`。星號不是引用運算子，它就是型別名稱的一部分。在裸指標的脈絡下，**不可變**代表指標不能在被解引用之後直接賦值。

和引用與智慧指標（smart pointer）不同，裸指標是：

* 允許忽略借用規則，同時可存在指向相同位置的可變和不可變的指標，或是多個可變指標
* 不能保證一定指向合法記憶體
* 可以為空（null）
* 並無實作任何自動清理機制

在停用 Rust 的保證之後，你能透過放棄這些安全性保證換得更高的效能，或是介接其他語言與硬體等無法套用 Rust 安全保證的場景。

範例 19-1 展示了如何從引用分別建立不可變和可變的裸指標。

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-01/src/main.rs:here}}
```

<span class="caption">範例 19-1：從引用建立裸指標</span>

注意，這段程式碼並無使用 `unsafe` 關鍵字。我們可以在安全程式碼中建立裸指標，我們只是不能在不安全區塊外對其解引用，你很快就會看到。

我們透過 `as` 將不可變與可變引用轉型成個別對應的裸指標。由於這些裸指標是從保證合法的引用而來，就能得知這些裸指標同樣合法，但我們無法推導所有裸指標都合法。

為了展示上述情形，接下來，我們將建立無法確認合法性的裸指標，範例 19-2 展示了如何從任意記憶體的位置建立裸指標。嘗試使用任意的記憶體行為並未定義，該位址上可能有也可能沒資料，且編譯器可能會最佳化該程式，所以該處可能不會存取記憶體，或是程式因區段錯誤導致崩潰。一般情況下，雖然這種程式碼能寫得出來，但不會有任何好理由寫出它。

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-02/src/main.rs:here}}
```

<span class="caption">範例 19-2：從任意記憶體位址建立裸指標</span>

回想一下，我們可以在安全的程式碼下建立裸指標，但我們不能對裸指標**解引用**並讀取它指向的資料。範例 19-3 我們對裸指標使用引用運算子 `*` 需要封裝在 `unsafe` 區塊內。

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-03/src/main.rs:here}}
```

<span class="caption">範例 19-3：在 `unsafe` 區塊對裸指標解引用</span>

建立一個指標沒有危險性，只有當我們嘗試存取它指向的值時，才可能需要處理非法的值。

請注意，範例 19-1 與 19-3，我們建立了 `*const i32` 與 `*mut i32` 兩個裸指標，皆指向相同儲存 `num` 的記憶體位置。若我們走正常程序建立指向 `num`  的不可變與可變引用，程式碼將因為 Rust 所有權規則不允許同時存在一個可變引用與多個不可變引用，進而無法編譯。有了裸指標，即可建立指向同個位置的可變指標和不可變指標，並透過可變指標改變其資料，但可能帶來資料競爭（data races），請小心！

既然有這些危險，為什麼你還要用裸指標呢？一個主要用例是與 C 程式碼介接，你將會在下一節[「呼叫不安全函式或方法」](#呼叫不安全函式或方法)讀到。另一個用例是在借用檢查器不理解之處建立一層安全抽象。我們將會介紹不安全函式，再探討一個使用到不安全程式碼的安全抽象範例。

### 呼叫不安全函式或方法

第二種需要不安全區塊的操作是呼叫不安全函式。不安全函式與方法外觀看起來與正常函式及方法並無二致，僅在整個函式定義前多了額外的 `unsafe` 。`unsafe` 關鍵字在此脈絡下是指此函式在呼叫時必須遵守某些要求，因為 Rust 無法保證我們能達成這項要求。當我們在一個 `unsafe` 區塊內呼叫一個 `unsafe` 函式，意味著我們已閱讀此函式的文件，而且有責任遵守此函式的使用條款。

這裡有個不安全函式叫做 `dangerous`，函式本體內無任何東西：

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/no-listing-01-unsafe-fn/src/main.rs:here}}
```

我們必須在單獨的 `unsafe` 區塊中呼叫 `dangerous` 函式，若不在 `unsafe` 區塊中呼叫，會得到一個錯誤：

```console
{{#include ../listings/ch19-advanced-features/output-only-01-missing-unsafe/output.txt}}
```

藉由一個 `unsafe` 區塊，我們可以對 Rust 聲明我們閱讀過該函式的文件，理解如何合理使用它，並且驗證過我們已履行該函式的使用條款。

不安全函式本體與 `unsafe` 區塊等效，所以可以在該不安全函式執行其他不安全操作，不需再加 `unsafe` 區塊。

#### 在不安全程式碼上建立安全的抽象

一個函式有不安全程式碼並不代表我們必須將整個函式標註為不安全。事實上，將不安全程式碼封裝在安全函式中一直是常見的抽象。我們來研讀標準函式庫的 `split_at_mut` 函式作為範例，它需要一些不安全程式碼。我們將探索如何實作之。這個安全方法定義在可變的切片上：它將一個切片在給定的索引引數（argument）上一分為二。範例 19-4 展示了如何使用 `split_at_mut`。

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-04/src/main.rs:here}}
```

<span class="caption">範例 19-4：使用一個安全的 `split_at_mut` 函式</span>

我們不可能在 safe Rust 下實作這個函式。一個嘗試可能會像範例 19-5 無法編譯。為了簡化，我們將 `split_at_mut` 實作為一個函式而非方法，並且以 `i32` 取代泛型型別 `T`。

```rust,ignore,does_not_compile
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-05/src/main.rs:here}}
```

<span class="caption">範例 19-5：嘗試僅用安全的 Rust 實作 `split_at_mut`</span>

這個函式先取得該切片的總長度，再來檢查從參數而來的索引小於等於該長度。這項檢查代表若我們傳入欲分割的索引位置大於該長度，這個函式會在嘗試使用該索引前就恐慌（panic）。

之後，我們回傳一個元組，其內包含兩個可變切片：一個從原始切片的起頭到 `mid` 索引位置，另一個則從 `mid` 到尾端。

當我們嘗試編譯範例 19-5 的程式碼，會得到一個錯誤。

```console
{{#include ../listings/ch19-advanced-features/listing-19-05/output.txt}}
```

Rust 的借用檢查器（borrow checker）不能理解我們同時借用一個切片的不同部分，它只認知到我們借用同一個切片兩次。借用同一個切片的不同部分基本上沒什麼問題，因為兩個切片不會重疊，但 Rust 不夠聰明以致無法理解這件事。當我們知道程式碼沒問題，但 Rust 並不知道，就是時候搞一點 不安全程式碼了。

範例 19-6 展示了如何使用一個 `unsafe` 區塊、一個裸指標，以及呼叫一些不安全函式來實作可成功執行的 `split_at_mut`

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-06/src/main.rs:here}}
```

<span class="caption">範例 19-6：在 `split_at_mut` 函式實作中使用不安全程式碼</span>

回憶一下第四章[「切片型別」][slice-型別] 一節中，提及切片會儲存指向某些資料的指標以及該切片長度。我們可使用 `len` 方法取得切片的長度，並用 `as_mut_ptr` 取得指向切片的裸指標。在此範例中，由於我們擁有指向某些 `i32` 值的可變切片，`as_mut_ptr` 會回傳一個型別為 `*mut i32` 的裸指標，即是儲存在 `ptr` 變數中的值。

我們判定 `mid` 索引在該切片內。此後我們進入不安全程式碼：`slice::from_raw_parts_mut` 函式需要一個裸指標與一個長度，並建立一個切片。我們使用這個函式來建立一個從 `ptr` 開始長度為 `mid` 的切片。而後，我們以 `mid` 作為引數，對 `ptr` 呼叫 `add` 方法，以取得從 `mid` 開始的裸指標，再來用此指標與從 `mid` 開始剩下的元素個數作為長度，建立另一個切片。

`slice::from_raw_parts_mut` 之所以為不安全函式，是因為它需要裸指標，且必須相信這個指標合法。`add` 是不安全方法是由於它必須相信偏移後的位址是合法指標。因此，我們需要在呼叫 `slice::from_raw_parts_mut` 和 `add` 外包一層 `unsafe` 函式。透過閱讀程式碼與加上對 `mid` 一定等於或比 `len` 小的斷言，我們可以宣稱所有在 `unsafe` 區塊的裸指標都是指向原始切片內的合法指標。這是一個可接受且合理的 `unsafe` 使用情境。

注意，我們不需替 `split_at_mut` 函式輸出結果做上 `unsafe` 的記號，而且我們可以在安全的 Rust 呼叫它。我們藉由安全的方式使用 `unsafe` 函式，完成了對不安全程式碼建立一層安全抽象，這個抽象只會從該函式能夠存取的資料內建立合法指標。

對比之下，範例 19-7 中使用 `slice::from_raw_parts_mut` 則極有可能會在該切片被使用時崩潰。這段程式碼從任意的記憶體位置建立了一個 10,000 元素長的切片。

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-07/src/main.rs:here}}
```

<span class="caption">範例 19-7：從任意記憶體位址建立切片</span>

我們不擁有此位址之下的記憶體，且並不保證這段程式碼建立的切片一定包含合法的 `i32` 值。嘗試將 `values` 當作合法的切片來使用，會導致為未定義行為（undefined behavior）。

#### 使用 `extern` 函式呼叫外部程式碼

有些時候，你的 Rust 程式碼可能需要與其他語言撰寫的程式碼互動。這種情況 Rust 提供 `extern` 關鍵字，予以協助建立與使用**外部函式介面（Foreign Function Interface，FFI）** 。FFI 的功能是給在一門程式語言定義函式，使得另一門（外部）程式語言可以呼叫這些函式。

範例 19-8 展示了如何建立整合一個 C 標準函式庫的 `abc` 函式。由於其他語言並無強制遵守 Rust 的規則和保證，而且 Rust 也無法檢查之，因此在 Rust 程式碼中呼叫在 `extern` 區塊內宣告的函式一定是不安全的操作，所以確保安全的重責大任就會落在程式設計師身上。

<span class="filename">檔案名稱：src/main.rs</span>

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-08/src/main.rs}}
```

<span class="caption">範例 19-8：宣告並呼叫一個用其他語言定義的 `extern` 函式</span>

在 `extern "C"` 區塊內，我們列出我們想要呼叫的，從其他語言而來的外部函式名稱與簽名。`"C"` 的部分定義了外部函式使用了哪個應用程式二進位制介面（ABI）：ABI 定義了在組合語言層級該如何呼叫此函式。`"C"` ABI 最為通用且遵循 C 程式語言的 ABI 規範。

> #### 從其他語言呼叫 Rust 函式
>
> 我們也可透過 `extern` 定義一個介面，允許其他語言呼叫 Rust 的函式。有別於建立整個 `extern` 區塊，我們會在 `fn` 關鍵字前加上 `extern` 關鍵字並指明應用程式二進位制介面（ABI）。我們甚至可加上 `#[no_mangle]` 註記來告訴編譯器不要重整（mangle）該函式名稱。**重整**是一個編譯器透過改變我們賦予函式的名稱，成為帶有更多資訊的名稱進而提供給編譯過程使用，但人類就相對難以閱讀。每個程式語言編譯器重整名稱的作法有些許不同，因此必須關閉 Rust 編譯器的名稱重整功能。
>
> 接下來的範例，我們寫了 `call_from_c` 函式，可以在編譯成共享函式庫（shared library）且連結至 C 後，由 C 程式碼存取：
>
> ```rust
> #[no_mangle]
> pub extern "C" fn call_from_c() {
>     println!("從 C 呼叫了一個 Rust 函式！");
> }
> ```
>
> 這類的 `extern` 用途不需要 `unsafe`。

### 存取或修改可變的靜態變數

在本書中，我們還沒聊到**全域變數**（global variable），這個 Rust 支援但會被 Rust 的所有權規則搞得七葷八素的功能。試想有兩個執行緒同時存取同一個可變全域變數，豈不導致資料競爭。

Rust 的全域變數稱做**靜態**變數。範例 19-9 展示了宣告並使用一個儲存字串切片的靜態變數。

<span class="filename">檔案名稱：src/main.rs</span>

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-09/src/main.rs}}
```

<span class="caption">範例 19-9：定義並使用一個不可變的靜態變數</span>


靜態變數（static variable）與我們在第三章「[變數與常數的差異][變數與常數的差異]<!-- ignore -->」一節討論的常數相似。慣例上靜態變數會用尖叫蛇式命名（`SCREAMING_SNAKE_CASE`）。由於靜態變數只能儲存 `static` 生命週期的引用，代表 Rust 編譯器可推導出它的生命週期，不需要我們顯式標註。存取一個不可變的靜態變數是安全的。

常數和不可變靜態變數看似相同，實則有些許隱晦差異：靜態變數之值有固定的記憶體位址，使用該值永遠會存取相同的資料。反之，常數在使用上則可複製它們儲存的資料。

另一個常數與靜態變數的差異是，靜態變數可能是可變的。存取並修改可變的靜態變數並「**不安全**」。範例 19-10 展示了如何宣告、存取、修改一個可變的靜態變數 `COUNTER`。

<span class="filename">檔案名稱：src/main.rs</span>

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-10/src/main.rs}}
```

<span class="caption">範例 19-10：讀取與寫入可變的靜態變數為不安全的操作</span>

與普通變數一樣，我們透過 `mut` 關鍵字指明可變性。任何讀寫 `COURTER` 的程式碼皆必須在 `unsafe` 區塊中。這個程式碼會編譯並打印出我們預期中的 `COUNTER: 3` 是因為他在單執行緒執行，若在多執行緒存取 `COUTER` 則可能導致資料競爭。

當能從全域存取可變資料時，確保沒有資料競爭就不容易了，這就是為什麼 Rust 將可變的靜態變數視為不安全。若是可能的話，我們推薦使用第十六章討論的並行技術與執行緒安全（thread-safe）的智慧指標（smart pointer），如此一來編譯器就能檢查從不同執行緒存取資料是安全的。

### 實作不安全特徵

我們可以用 `unsafe` 實作不安全特徵。當一個特徵是有至少一個方法包含編譯器無法驗證的不變條件（invariant），就稱該特徵不安全。我們透過在 `trait` 前加上 `unsafe` 關鍵字來宣告一個特徵為 `unsafe`，這也讓實作該特徵會變成 `unsafe`，如 19-11 所示。

```rust
{{#rustdoc_include ../listings/ch19-advanced-features/listing-19-11/src/main.rs}}
```

<span class="caption">範例 19-11：定義並實作一個不安全特徵</span>

透過 `unsafe impl`，我們承諾我們將會遵守這些編譯器無法驗證的不變條件（invariant）。

回想第十六章[「可延展的並行與 `Sync` 及 `Send` 特徵」][可延展的並行與 `Sync` 及 `Send` 特徵]一節的兩個記號特徵（marker trait）`Sync` 與 `Send`：若我們的型別是由 `Send` 與 `Sync` 組合而成，編譯器會自動實作這些特徵。若我們的型別包含一些非 `Send` 或 `Sync` 的型別，例如裸指標，但我們希望替型別做上 `Send` 或 `Sync` 的記號，就必須使用 `unsafe`。Rust 無法驗證我們的型別有遵守可以在多執行緒中傳遞或存取的保證。因而，我們需要自己手動檢查，並指明這是 `unsafe`。

### 存取聯合體的欄位

最後一個可以使用 `unsafe` 的地方是存取 `union` 的欄位。`union` 與 `struct` 十分相似，差異是在一個聯合體實例中僅儲存其中一個宣告的欄位。聯合體主要用在與 C 程式碼的聯合體介接。存取聯合體的欄位並不安全，由於 Rust 無法保證當前儲存在聯合體實例中的資料是什麼型別，因此存取聯合體的欄位並不安全。你可以從 [Rust 參考手冊][參考手冊]了解更多關於聯合體的資訊。

### 何時該用不安全程式碼

透過 `unsafe` 使用上述五種功能（超能力）並沒有錯，更並非不能接受，但由於編譯期無法協助遵守記憶體安全，這讓 `unsafe` 程式碼要正確無誤略顯棘手。當你因故需要使用 `unsafe` 程式碼，就去用吧，並且記得替 `unsafe` 撰寫明確的註釋，讓有問題發生時更容易追蹤查找源頭。

[迷途引用]: ch04-02-references-and-borrowing.html#迷途引用
[變數與常數的差異]: ch03-01-variables-and-mutability.html#常數
[可延展的並行與 `Sync` 及 `Send` 特徵]: ch16-04-extensible-concurrency-sync-and-send.html#可延展的並行與-sync-及-send-特徵
[slice-型別]: ch04-03-slices.html#切片型別
[參考手冊]: https://doc.rust-lang.org/reference/items/unions.html
