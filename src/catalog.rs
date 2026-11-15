use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub const WEBSITE: &str = "https://sunrise-website-orcin.vercel.app";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Image,
    Video,
    Illustration,
    Documents,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Self::Image => "Imagem",
            Self::Video => "Vídeo",
            Self::Illustration => "Ilustração",
            Self::Documents => "Documentos",
        }
    }
}

pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub scope: &'static [&'static str],
    pub cover: &'static [u8],
    pub aero_cover: &'static [u8],
    pub icon: &'static [u8],
}

impl Product {
    pub fn matches(&self, query: &str, category: Option<Category>) -> bool {
        category.is_none_or(|value| value == self.category)
            && search_key(&format!(
                "{} {} {} {}",
                self.name,
                self.category.label(),
                self.subtitle,
                self.description
            ))
            .contains(&search_key(query.trim()))
    }
}

fn search_key(text: &str) -> String {
    text.nfd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .collect()
}

macro_rules! product {
    ($id:literal, $name:literal, $category:ident, $subtitle:literal, $description:literal, [$($scope:literal),+]) => {
        Product {
            id: $id,
            name: $name,
            category: Category::$category,
            subtitle: $subtitle,
            description: $description,
            scope: &[$($scope),+],
            cover: include_bytes!(concat!("../assets/", $id, ".webp")),
            aero_cover: include_bytes!(concat!("../assets/aero-light/", $id, ".webp")),
            icon: include_bytes!(concat!("../assets/aero-icons/", $id, "-192.png")),
        }
    };
}

pub static PRODUCTS: [Product; 7] = [
    product!(
        "risephoto",
        "RisePhoto",
        Image,
        "Imagem e composição",
        "Edite imagens, pinte e componha com camadas e máscaras.",
        [
            "Composição em camadas",
            "Máscaras, seleções e pincéis",
            "Edição não destrutiva"
        ]
    ),
    product!(
        "risefilm",
        "RiseFilm",
        Video,
        "Edição de vídeo",
        "Monte histórias em vídeo, organize trilhas e dê ritmo aos seus cortes.",
        [
            "Timeline de vídeo e áudio",
            "Cortes, transições e prévia",
            "Exportação acelerada onde suportada"
        ]
    ),
    product!(
        "risevector",
        "RiseVector",
        Illustration,
        "Ilustração vetorial",
        "Desenhe formas que mantêm a precisão em qualquer escala.",
        ["Curvas e formas editáveis", "Ilustração e design vetorial"]
    ),
    product!(
        "risedocs",
        "RiseDocs",
        Documents,
        "PDF e documentos",
        "Leia, anote e organize PDFs em um aplicativo local.",
        [
            "Leitura e anotação",
            "Organização de páginas",
            "Ferramentas offline"
        ]
    ),
    product!(
        "risemotion",
        "RiseMotion",
        Video,
        "Motion e efeitos",
        "Crie movimento com quadros-chave, efeitos e composição de cenas.",
        [
            "Animação por quadros-chave",
            "Composição de camadas e efeitos"
        ]
    ),
    product!(
        "riseluma",
        "RiseLuma",
        Image,
        "Fotografia e RAW",
        "Organize sua biblioteca e revele arquivos RAW com ajustes não destrutivos.",
        [
            "Catálogo fotográfico",
            "Revelação de RAW",
            "Ajustes em lote"
        ]
    ),
    product!(
        "risedesign",
        "RiseDesign",
        Illustration,
        "Design gráfico rápido",
        "Crie peças gráficas para o dia a dia com um fluxo direto e acessível.",
        [
            "Criação de materiais gráficos",
            "Composição e edição visual simplificadas"
        ]
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_seven_unique_products_and_readable_assets() {
        assert_eq!(PRODUCTS.len(), 7);
        let ids = PRODUCTS
            .iter()
            .map(|product| product.id)
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), PRODUCTS.len());
        for product in &PRODUCTS {
            assert!(
                image::load_from_memory(product.cover).is_ok(),
                "{}",
                product.id
            );
            assert!(
                image::load_from_memory(product.aero_cover).is_ok(),
                "{} (Aero)",
                product.id
            );
            assert!(
                image::load_from_memory(product.icon).is_ok(),
                "{}",
                product.id
            );
        }
    }

    #[test]
    fn search_matches_accentless_keywords_and_combines_category() {
        assert!(PRODUCTS[1].matches("  VIDEO", Some(Category::Video)));
        assert!(!PRODUCTS[1].matches("video", Some(Category::Image)));
        assert!(PRODUCTS[2].matches("ilustracao", None));
        assert!(
            !PRODUCTS
                .iter()
                .any(|p| p.matches("sem correspondência", None))
        );
    }
}
