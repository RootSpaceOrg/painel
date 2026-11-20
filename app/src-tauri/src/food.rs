// Busca de alimentos no Open Food Facts (base aberta, sem chave). Valores por 100 g/ml.
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Serialize)]
pub struct FoodHit {
    name: String,
    brand: String,
    kcal: f64,
    p: f64,
    c: f64,
    f: f64,
}

fn num(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.replace(',', ".").parse().ok()))
}

fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn search(query: &str) -> Result<Vec<FoodHit>, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).user_agent("Painel/0.1").build();
    let res: Value = agent
        .get("https://br.openfoodfacts.org/cgi/search.pl")
        .query("search_terms", query)
        .query("search_simple", "1")
        .query("action", "process")
        .query("json", "1")
        .query("page_size", "25")
        .query("fields", "product_name,product_name_pt,brands,nutriments")
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for p in res["products"].as_array().into_iter().flatten() {
        let n = &p["nutriments"];
        // só produtos com calorias informadas (kcal direto ou kJ convertido)
        let kcal = num(&n["energy-kcal_100g"]).or_else(|| num(&n["energy_100g"]).map(|kj| kj / 4.184));
        let name = p["product_name_pt"].as_str().filter(|s| !s.is_empty()).or(p["product_name"].as_str()).unwrap_or("").trim();
        let (Some(kcal), false) = (kcal, name.is_empty()) else { continue };
        out.push(FoodHit {
            name: name.to_string(),
            brand: p["brands"].as_str().unwrap_or("").split(',').next().unwrap_or("").trim().to_string(),
            kcal: kcal.round(),
            p: r1(num(&n["proteins_100g"]).unwrap_or(0.0)),
            c: r1(num(&n["carbohydrates_100g"]).unwrap_or(0.0)),
            f: r1(num(&n["fat_100g"]).unwrap_or(0.0)),
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn food_search(query: String) -> Result<Vec<FoodHit>, String> {
    tauri::async_runtime::spawn_blocking(move || search(&query)).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    // rede real: cargo test off_search -- --ignored --nocapture
    #[test]
    #[ignore]
    fn off_search() {
        for h in super::search("whey").expect("busca").iter().take(5) {
            println!("{} ({}) {} kcal P{} C{} G{}", h.name, h.brand, h.kcal, h.p, h.c, h.f);
        }
    }
}
