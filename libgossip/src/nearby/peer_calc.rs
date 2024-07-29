use std::collections::HashMap;
use crate::ble::{AddressData, DocumentData, PeerData};
use crate::ble::PeerState::Settled;
use crate::data::UUID;

pub fn find_best_doc_from_peers(my_exchange_doc: &DocumentData, peers: &HashMap<UUID, PeerData>) -> DocumentData {
    let (non_scan_peers , scan_peers): (Vec<&PeerData>, Vec<&PeerData>) = peers.values().partition(|&n| n.peer_state == Settled );

//    println!("nonscanning peers {:?}", non_scan_peers);
//    println!("scannign peers {:?}", scan_peers);

    let non_scan_docs: Vec<&Vec<u8>> = non_scan_peers.iter().map(|n|&n.document_data).collect();
    let mut scan_docs: Vec<&Vec<u8>> = scan_peers.iter().map(|n|&n.document_data).collect();


    let non_scan_docs_by_count = create_counts(&non_scan_peers);
    let most_freq = get_most_freq_doc(&non_scan_docs_by_count);

    // check the non-scanning folks first
    if let Some((doc, cnt)) = most_freq {
        return if cnt > 1 {
            println!("gonna return a nonscanner with multiple ppl!");
            doc.clone()
        } else {
            println!("gonna return a nonscanner that is smalleset");
            // there is no most frequent - pick the smallest valued
            let smallest = smallest(non_scan_docs.iter());
            smallest.unwrap().clone()
        }
    }

    //check the scanning
    scan_docs.push(my_exchange_doc);
    println!("gonna return regular smallest scanner");
    let desired_doc = smallest(scan_docs.iter());
    desired_doc.unwrap().clone()
}


fn create_counts<'a>(peers: &'a Vec<&PeerData>) -> HashMap<&'a Vec<u8>, u16> {
    let mut map = HashMap::new();
    for p in peers {
        if let Some(cnt) = map.get(&p.document_data) {
            map.insert(&p.document_data, cnt+1);
        } else {
            map.insert(&p.document_data, 1);
        }
    }
    map
}

fn get_most_freq_doc<'a>(doc_counts: &'a HashMap<&Vec<u8>, u16>) -> Option<(&'a Vec<u8>, u16)> {
    let biggest = doc_counts.iter().max_by(|a,b| a.1.cmp(b.1));
    if let Some(z) = biggest {
        Some((*z.0, *z.1))
    } else {
        None
    }
}

fn smallest<'a, I>(iter: I) -> Option<&'a Vec<u8>>
    where
        I: Iterator<Item = &'a&'a Vec<u8>>
{
    let z = iter.min_by(|a, b| a.cmp(b));

    if let Some(z) = z {
        Some(*z)
    } else {
        None
    }
}

pub fn collect_addrs_for_doc<'a, 'b, I>(doc: &'b DocumentData, peers: I) -> Vec<AddressData>
    where
        I: Iterator<Item = &'a PeerData>
{
    let result: Vec<Vec<u8>> = peers.filter(|p| &p.document_data == doc)
        .map(|m| m.address_data.clone())
        .collect();
    result
}
