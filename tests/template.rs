use std::time::Instant;

use boom::boom::Match;
use boom::boom::parse_templates::parse_template_indexes;

#[test]
fn test_empty_template() {
    let template = "https://trika.gay";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!("Took {:?} to get template indices (EMPTY)", timer.elapsed());
    assert_eq!(indices, None);
}

#[test]
fn test_template_suffix() {
    let template = "https://google.com/search?q={{{s}}}";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (SUFFIX)",
        timer.elapsed()
    );
    assert_eq!(
        indices,
        Some([
            Match::new(template.len() - "{{{s}}}".len(), template.len()),
            Match::new(0, 0)
        ])
    );
}

#[test]
fn test_template_suffix_long() {
    let template = "http://shop.zuckerzauber.at/epages/es121414.sf/de_AT/?ObjectPath=/Shops/es121414_Caros_Zuckerzauber&ViewAction=DetailSearchProducts&Search=SF-AllStrings&SearchString={{{s}}}";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (SUFFIX LONG)",
        timer.elapsed()
    );
    assert_eq!(
        indices,
        Some([
            Match::new(template.len() - "{{{s}}}".len(), template.len()),
            Match::new(0, 0)
        ])
    );
}

#[test]
fn test_template_infix() {
    let template = "http://www.db.yugioh-card.com/{{{s}}}/card_search.action";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!("Took {:?} to get template indices (INFIX)", timer.elapsed());
    assert_eq!(indices, Some([Match::new(30, 37), Match::new(0, 0)]));
}

#[test]
fn test_template_infix_long() {
    let template = "http://www.db.yugioh-card.com/yugiohdb/card_search.action?ope=1&sess=1&keyword={{{s}}}&stype=1&ctype=&starfr=&starto=&atkfr=&atkto=&deffr=&defto=&othercon=1";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (INFIX LONG)",
        timer.elapsed()
    );
    assert_eq!(indices, Some([Match::new(79, 86), Match::new(0, 0)]));
}

#[test]
fn test_template_infix_multiple() {
    let template = "http://www.db.yugioh-card.com/{{{s}}}/card_search.action?ope=1&sess=1&keyword={{{s}}}&stype=1&ctype=&starfr=&starto=&atkfr=&atkto=&deffr=&defto=&othercon=1";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (INFIX LONG MULTIPLE)",
        timer.elapsed()
    );
    assert_eq!(indices, Some([Match::new(30, 37), Match::new(78, 85)]));
}

#[cfg(feature = "measure-allocs")]
mod tests {
    use super::*;

    #[test]
    fn test_empty_template_memory() {
        let alloc = allocation_counter::measure(|| {
            test_empty_template();
        });
        eprintln!(
            "`test_empty_template` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_template_infix_long_memory() {
        let alloc = allocation_counter::measure(|| {
            test_template_infix_long();
        });
        eprintln!(
            "`test_template_infix_long` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_template_suffix_long_memory() {
        let alloc = allocation_counter::measure(|| {
            test_template_suffix_long();
        });
        eprintln!(
            "`test_template_suffix_long` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }
}
